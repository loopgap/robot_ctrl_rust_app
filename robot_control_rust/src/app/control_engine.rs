use crate::models::*;

macro_rules! algorithm_accessor {
    ($name:ident, $name_mut:ident, $type:ty, $idx:expr) => {
        pub fn $name(&self) -> &$type {
            self.algorithms[$idx]
                .as_any()
                .downcast_ref::<$type>()
                .expect(concat!(stringify!($type), " at index ", stringify!($idx)))
        }
        pub fn $name_mut(&mut self) -> &mut $type {
            self.algorithms[$idx]
                .as_any_mut()
                .downcast_mut::<$type>()
                .expect(concat!(stringify!($type), " at index ", stringify!($idx)))
        }
    };
}

/// 控制算法引擎，管理 10 种控制算法的统一接口
///
/// 支持的算法：PID、增量PID、Bang-Bang、模糊PID、串级PID、
/// Smith预估、ADRC、LADRC、LQR、MPC
///
/// 通过 `algorithms` 向量存储所有算法，`active_index` 跟踪当前激活的算法。
pub struct ControlEngine {
    pub algorithms: Vec<Box<dyn ControlAlgorithm>>,
    pub active_index: usize,
    pub current_state: RobotState,
    pub state_history: Vec<RobotState>,
    pub is_running: bool,
    pub presets: Vec<Preset>,
    pub topology: TopologyConfig,
    pub builtin_topologies: Vec<TopologyConfig>,
    pub nn: NeuralNetwork,
    pub nn_suggested_kp: f64,
    pub nn_suggested_ki: f64,
    pub nn_suggested_kd: f64,
}

impl ControlEngine {
    pub fn new() -> Self {
        Self {
            algorithms: vec![
                Box::new(PidController::new(1.0, 0.1, 0.01, 0.0)),
                Box::new(IncrementalPidController::new(1.0, 0.1, 0.01, 0.0)),
                Box::new(BangBangController::new(0.0, 1.0, -1.0, 0.5)),
                Box::new(FuzzyPidController::new(1.0, 0.1, 0.01, 0.0)),
                Box::new(CascadePidController::new(
                    1.0, 0.1, 0.01, 100.0, 1.0, 0.1, 0.01, 100.0, 0.0,
                )),
                Box::new(SmithPredictorController::new(1.0, 0.1, 0.01, 0.0)),
                Box::new(AdrcController::new(0.0, 1.0)),
                Box::new(LadrcController::new(0.0, 10.0, 100.0, 1.0)),
                Box::new(LqrController::new(0.0, 1.0, 1.0, 0.1)),
                Box::new(MpcController::new(0.0, 10, 3)),
            ],
            active_index: 0,
            current_state: RobotState::default(),
            state_history: Vec::new(),
            is_running: false,
            presets: Vec::new(),
            topology: TopologyConfig::default(),
            builtin_topologies: TopologyConfig::builtin_list(),
            nn: NeuralNetwork::pid_tuner(),
            nn_suggested_kp: 0.0,
            nn_suggested_ki: 0.0,
            nn_suggested_kd: 0.0,
        }
    }

    pub fn active_algorithm(&self) -> &dyn ControlAlgorithm {
        self.algorithms[self.active_index].as_ref()
    }

    pub fn active_algorithm_mut(&mut self) -> &mut dyn ControlAlgorithm {
        self.algorithms[self.active_index].as_mut()
    }

    pub fn set_active_algorithm(&mut self, index: usize) {
        if index < self.algorithms.len() {
            self.active_index = index;
        }
    }

    pub fn algorithm_name(&self) -> &'static str {
        self.active_algorithm().name()
    }

    pub fn compute(&mut self, feedback: f64) -> f64 {
        self.active_algorithm_mut().compute(feedback)
    }

    pub fn compute_dual(&mut self, position: f64, velocity: f64) -> f64 {
        if let Some(dual) = self
            .active_algorithm_mut()
            .as_any_mut()
            .downcast_mut::<CascadePidController>()
        {
            return dual.compute_dual(position, velocity);
        }
        if let Some(dual) = self
            .active_algorithm_mut()
            .as_any_mut()
            .downcast_mut::<LqrController>()
        {
            return dual.compute_dual(position, velocity);
        }
        self.active_algorithm_mut().compute(position)
    }

    pub fn setpoint(&self) -> f64 {
        self.active_algorithm().setpoint()
    }

    pub fn set_setpoint(&mut self, sp: f64) {
        self.active_algorithm_mut().set_setpoint(sp);
    }

    pub fn output(&self) -> f64 {
        self.active_algorithm().output()
    }

    pub fn reset_active(&mut self) {
        self.active_algorithm_mut().reset();
    }

    pub fn toggle_running(&mut self) {
        self.is_running = !self.is_running;
    }

    pub fn emergency_stop(&mut self) {
        self.is_running = false;
        self.set_setpoint(0.0);
        self.reset_active();
    }

    pub fn push_state(&mut self, state: RobotState) {
        self.state_history.push(state);
        const MAX_HISTORY: usize = 10000;
        if self.state_history.len() > MAX_HISTORY {
            // Use drain(..1) instead of remove(0) — both are O(n) for Vec, but
            // drain avoids the return-value overhead and is idiomatic for
            // "discard oldest N elements".
            self.state_history.drain(..1);
        }
    }

    pub fn nn_train_step(&mut self) -> Option<String> {
        let errors: Vec<f64> = self.state_history.iter().map(|s| s.error).collect();
        if errors.len() < 20 {
            return None;
        }
        let features = NeuralNetwork::extract_features(&errors);
        let performance =
            1.0 / (1.0 + errors.iter().map(|e| e.abs()).sum::<f64>() / errors.len() as f64);
        let pid = self.pid();
        let target = vec![
            (pid.kp / 5.0).clamp(0.0, 1.0) * performance,
            (pid.ki / 2.0).clamp(0.0, 1.0) * performance,
            (pid.kd / 1.0).clamp(0.0, 1.0) * performance,
        ];
        let loss = self.nn.train_step(&features, &target);
        Some(format!(
            "NN Training - Loss: {:.6}, Epoch: {}",
            loss, self.nn.training_epochs
        ))
    }

    pub fn nn_suggest_params(&mut self) {
        let errors: Vec<f64> = self.state_history.iter().map(|s| s.error).collect();
        if errors.len() < 10 {
            return;
        }
        let features = NeuralNetwork::extract_features(&errors);
        let output = self.nn.forward(&features);
        self.nn_suggested_kp = output[0] * 5.0;
        self.nn_suggested_ki = output[1] * 2.0;
        self.nn_suggested_kd = output[2] * 1.0;
    }

    pub fn apply_nn_params(&mut self) {
        let kp = self.nn_suggested_kp;
        let ki = self.nn_suggested_ki;
        let kd = self.nn_suggested_kd;
        let pid = self.pid_mut();
        pid.kp = kp;
        pid.ki = ki;
        pid.kd = kd;
    }

    algorithm_accessor!(pid, pid_mut, PidController, 0);
    algorithm_accessor!(
        incremental_pid,
        incremental_pid_mut,
        IncrementalPidController,
        1
    );
    algorithm_accessor!(bang_bang, bang_bang_mut, BangBangController, 2);
    algorithm_accessor!(fuzzy_pid, fuzzy_pid_mut, FuzzyPidController, 3);
    algorithm_accessor!(cascade_pid, cascade_pid_mut, CascadePidController, 4);
    algorithm_accessor!(
        smith_predictor,
        smith_predictor_mut,
        SmithPredictorController,
        5
    );
    algorithm_accessor!(adrc, adrc_mut, AdrcController, 6);
    algorithm_accessor!(ladrc, ladrc_mut, LadrcController, 7);
    algorithm_accessor!(lqr, lqr_mut, LqrController, 8);
    algorithm_accessor!(mpc, mpc_mut, MpcController, 9);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::DualFeedbackControl;

    #[test]
    fn test_new_creates_all_algorithms() {
        let engine = ControlEngine::new();
        assert_eq!(engine.algorithms.len(), 10);
        // Verify all expected names
        let expected_names = [
            "Classic PID",
            "Incremental PID",
            "Bang-Bang",
            "Fuzzy PID",
            "Cascade PID",
            "Smith Predictor",
            "ADRC",
            "LADRC",
            "LQR",
            "MPC",
        ];
        for (i, expected) in expected_names.iter().enumerate() {
            let mut e = ControlEngine::new();
            e.set_active_algorithm(i);
            assert_eq!(e.algorithm_name(), *expected, "algo[{}]", i);
        }
    }

    #[test]
    fn test_set_active_algorithm_valid_and_invalid() {
        let mut engine = ControlEngine::new();
        engine.set_active_algorithm(6);
        assert_eq!(engine.algorithm_name(), "ADRC");
        // Out of range ignored
        engine.set_active_algorithm(999);
        assert_eq!(engine.active_index, 6);
    }

    #[test]
    fn test_compute_and_output_consistency() {
        let mut engine = ControlEngine::new();
        engine.set_setpoint(10.0);
        let output = engine.compute(5.0);
        assert!(output.is_finite());
        assert_eq!(engine.output(), engine.active_algorithm().output());
    }

    #[test]
    fn test_setpoint_roundtrip() {
        let mut engine = ControlEngine::new();
        assert_eq!(engine.setpoint(), 0.0);
        engine.set_setpoint(42.0);
        assert_eq!(engine.setpoint(), 42.0);
        engine.set_setpoint(-100.0);
        assert_eq!(engine.setpoint(), -100.0);
    }

    #[test]
    fn test_toggle_and_emergency_stop() {
        let mut engine = ControlEngine::new();
        assert!(!engine.is_running);
        engine.toggle_running();
        assert!(engine.is_running);
        engine.set_setpoint(50.0);
        engine.compute(1.0);
        engine.emergency_stop();
        assert!(!engine.is_running);
        assert_eq!(engine.setpoint(), 0.0);
        assert_eq!(engine.output(), 0.0);
    }

    #[test]
    fn test_push_state_preserves_order_and_limits() {
        let mut engine = ControlEngine::new();
        for i in 0..10001 {
            engine.push_state(RobotState {
                position: i as f64,
                ..Default::default()
            });
        }
        assert_eq!(engine.state_history.len(), 10000);
        // Oldest entry (0.0) was evicted, newest starts at 1.0
        assert_eq!(engine.state_history.first().unwrap().position, 1.0);
        assert_eq!(engine.state_history.last().unwrap().position, 10000.0);
    }

    #[test]
    fn test_all_algorithms_finite_output() {
        let mut engine = ControlEngine::new();
        for i in 0..engine.algorithms.len() {
            engine.set_active_algorithm(i);
            let output = engine.compute(1.0);
            assert!(
                output.is_finite(),
                "{} produced non-finite",
                engine.algorithm_name()
            );
        }
    }

    #[test]
    fn test_all_algorithms_reset_to_zero() {
        let mut engine = ControlEngine::new();
        for i in 0..engine.algorithms.len() {
            engine.set_active_algorithm(i);
            engine.compute(1.0);
            engine.reset_active();
            assert_eq!(engine.output(), 0.0, "{}", engine.algorithm_name());
        }
    }

    #[test]
    fn test_compute_dual_cascade_pid() {
        let mut engine = ControlEngine::new();
        engine.set_active_algorithm(4); // Cascade PID
        let out = engine.compute_dual(10.0, 0.5);
        assert!(out.is_finite());
    }

    #[test]
    fn test_compute_dual_lqr() {
        let mut engine = ControlEngine::new();
        engine.set_active_algorithm(8); // LQR
        let out = engine.compute_dual(10.0, 0.5);
        assert!(out.is_finite());
    }

    #[test]
    fn test_compute_dual_fallback_for_non_dual_algo() {
        let mut engine = ControlEngine::new();
        engine.set_active_algorithm(0); // Classic PID — no dual
        let dual_out = engine.compute_dual(5.0, 1.0);
        // Falls back to compute(position)
        let mut engine2 = ControlEngine::new();
        let single_out = engine2.compute(5.0);
        // Both should produce finite values (not necessarily equal due to state)
        assert!(dual_out.is_finite());
        assert!(single_out.is_finite());
    }

    #[test]
    fn test_topology_defaults() {
        let engine = ControlEngine::new();
        assert!(!engine.builtin_topologies.is_empty());
    }

    #[test]
    fn test_dual_feedback_trait_cascade_pid() {
        let mut engine = ControlEngine::new();
        engine.set_active_algorithm(4);
        let algorithm = engine.active_algorithm_mut();
        let dual = algorithm
            .as_any_mut()
            .downcast_mut::<CascadePidController>()
            .expect("CascadePidController");
        let output = dual.compute_dual(10.0, 1.0);
        assert!(output.is_finite());
    }

    #[test]
    fn test_dual_feedback_trait_lqr() {
        let mut engine = ControlEngine::new();
        engine.set_active_algorithm(8);
        let algorithm = engine.active_algorithm_mut();
        let dual = algorithm
            .as_any_mut()
            .downcast_mut::<LqrController>()
            .expect("LqrController");
        let output = dual.compute_dual(10.0, 1.0);
        assert!(output.is_finite());
    }

    // ── Deep: pid accessor macros ──

    #[test]
    fn test_pid_accessor() {
        let engine = ControlEngine::new();
        let pid = engine.pid();
        assert_eq!(pid.kp, 1.0);
        assert_eq!(pid.ki, 0.1);
        assert_eq!(pid.kd, 0.01);
    }

    #[test]
    fn test_pid_mut_accessor() {
        let mut engine = ControlEngine::new();
        engine.pid_mut().kp = 5.0;
        assert_eq!(engine.pid().kp, 5.0);
    }

    #[test]
    fn test_incremental_pid_accessor() {
        let mut engine = ControlEngine::new();
        engine.incremental_pid_mut().kp = 2.5;
        assert_eq!(engine.incremental_pid().kp, 2.5);
    }

    #[test]
    fn test_bang_bang_accessor() {
        let mut engine = ControlEngine::new();
        engine.bang_bang_mut().set_setpoint(5.0);
        let bb = engine.bang_bang();
        assert_eq!(bb.setpoint(), 5.0);
        assert!(bb.as_any().downcast_ref::<BangBangController>().is_some());
    }

    #[test]
    fn test_fuzzy_pid_accessor() {
        let mut engine = ControlEngine::new();
        assert_eq!(engine.fuzzy_pid().kp_base, 1.0);
        engine.fuzzy_pid_mut().kp_base = 2.5;
        assert_eq!(engine.fuzzy_pid().kp_base, 2.5);
    }

    #[test]
    fn test_cascade_pid_accessor() {
        let mut engine = ControlEngine::new();
        let cp = engine.cascade_pid();
        assert!(cp.as_any().downcast_ref::<CascadePidController>().is_some());
        assert_eq!(cp.setpoint(), 0.0);
        engine.cascade_pid_mut().set_setpoint(10.0);
        assert_eq!(engine.cascade_pid().setpoint(), 10.0);
    }

    #[test]
    fn test_smith_predictor_accessor() {
        let mut engine = ControlEngine::new();
        let sp = engine.smith_predictor();
        assert!(sp
            .as_any()
            .downcast_ref::<SmithPredictorController>()
            .is_some());
        engine.smith_predictor_mut().set_setpoint(7.0);
        assert_eq!(engine.smith_predictor().setpoint(), 7.0);
    }

    #[test]
    fn test_adrc_accessor() {
        let mut engine = ControlEngine::new();
        let adrc = engine.adrc();
        assert!(adrc.as_any().downcast_ref::<AdrcController>().is_some());
        engine.adrc_mut().set_setpoint(3.0);
        assert_eq!(engine.adrc().setpoint(), 3.0);
    }

    #[test]
    fn test_ladrc_accessor() {
        let mut engine = ControlEngine::new();
        let ladrc = engine.ladrc();
        assert!(ladrc.as_any().downcast_ref::<LadrcController>().is_some());
        engine.ladrc_mut().set_setpoint(4.0);
        assert_eq!(engine.ladrc().setpoint(), 4.0);
    }

    #[test]
    fn test_lqr_accessor() {
        let mut engine = ControlEngine::new();
        let lqr = engine.lqr();
        assert!(lqr.as_any().downcast_ref::<LqrController>().is_some());
        engine.lqr_mut().set_setpoint(6.0);
        assert_eq!(engine.lqr().setpoint(), 6.0);
    }

    #[test]
    fn test_mpc_accessor() {
        let mut engine = ControlEngine::new();
        let mpc = engine.mpc();
        assert!(mpc.as_any().downcast_ref::<MpcController>().is_some());
        engine.mpc_mut().set_setpoint(8.0);
        assert_eq!(engine.mpc().setpoint(), 8.0);
    }

    // ── Deep: NN training pipeline ──

    #[test]
    fn test_nn_train_step_insufficient_history() {
        let mut engine = ControlEngine::new();
        // Push fewer than 20 states
        for i in 0..15 {
            engine.push_state(RobotState {
                error: (i as f64 * 0.1).sin(),
                ..Default::default()
            });
        }
        assert!(engine.nn_train_step().is_none(), "need >= 20 states");
    }

    #[test]
    fn test_nn_train_step_sufficient_history() {
        let mut engine = ControlEngine::new();
        for i in 0..25 {
            engine.push_state(RobotState {
                error: (i as f64 * 0.1).sin(),
                ..Default::default()
            });
        }
        let result = engine.nn_train_step();
        assert!(result.is_some());
        let msg = result.unwrap();
        assert!(msg.contains("Loss"));
        assert!(msg.contains("Epoch"));
    }

    #[test]
    fn test_nn_train_step_increments_epochs() {
        let mut engine = ControlEngine::new();
        for i in 0..25 {
            engine.push_state(RobotState {
                error: (i as f64 * 0.1).sin(),
                ..Default::default()
            });
        }
        engine.nn_train_step();
        let epoch_after_first = engine.nn.training_epochs;
        engine.nn_train_step();
        assert!(engine.nn.training_epochs > epoch_after_first);
    }

    #[test]
    fn test_nn_suggest_params_insufficient_history() {
        let mut engine = ControlEngine::new();
        for i in 0..5 {
            engine.push_state(RobotState {
                error: i as f64,
                ..Default::default()
            });
        }
        // Should be no-op with < 10 states
        engine.nn_suggest_params();
        assert_eq!(engine.nn_suggested_kp, 0.0);
    }

    #[test]
    fn test_nn_suggest_params_sufficient_history() {
        let mut engine = ControlEngine::new();
        for i in 0..15 {
            engine.push_state(RobotState {
                error: (i as f64 * 0.2).sin(),
                ..Default::default()
            });
        }
        engine.nn_suggest_params();
        // Suggested params should be finite (may be zero depending on NN output)
        assert!(engine.nn_suggested_kp.is_finite());
        assert!(engine.nn_suggested_ki.is_finite());
        assert!(engine.nn_suggested_kd.is_finite());
    }

    #[test]
    fn test_apply_nn_params() {
        let mut engine = ControlEngine::new();
        engine.nn_suggested_kp = 2.5;
        engine.nn_suggested_ki = 0.25;
        engine.nn_suggested_kd = 0.025;
        engine.apply_nn_params();
        let pid = engine.pid();
        assert_eq!(pid.kp, 2.5);
        assert_eq!(pid.ki, 0.25);
        assert_eq!(pid.kd, 0.025);
    }

    // ── Deep: different algorithms produce different outputs ──

    #[test]
    fn test_different_algorithms_produce_different_outputs() {
        let mut outputs = Vec::new();
        for i in 0..10 {
            let mut engine = ControlEngine::new();
            engine.set_active_algorithm(i);
            engine.set_setpoint(10.0);
            // Feed same error multiple times for stateful algorithms
            let mut last_output = 0.0;
            for _ in 0..5 {
                last_output = engine.compute(5.0);
            }
            outputs.push((engine.algorithm_name(), last_output));
        }
        // At least some outputs should differ
        let unique: std::collections::HashSet<i64> = outputs
            .iter()
            .map(|(_, o)| (o * 1000.0).round() as i64)
            .collect();
        assert!(
            unique.len() > 1,
            "algorithms should produce different outputs: {:?}",
            outputs
        );
    }

    // ── Deep: presets and NN defaults ──

    #[test]
    fn test_defaults() {
        let engine = ControlEngine::new();
        assert!(engine.presets.is_empty());
        assert_eq!(engine.nn_suggested_kp, 0.0);
        assert_eq!(engine.nn_suggested_ki, 0.0);
        assert_eq!(engine.nn_suggested_kd, 0.0);
        assert!(!engine.is_running);
        assert_eq!(engine.active_index, 0);
        assert!(engine.state_history.is_empty());
    }

    #[test]
    fn test_nn_is_pid_tuner() {
        let engine = ControlEngine::new();
        // NeuralNetwork::pid_tuner creates [6, 16, 8, 3] = 3 layers
        assert_eq!(engine.nn.layers.len(), 3);
        assert_eq!(engine.nn.layers[0].biases.len(), 16); // first hidden
        assert_eq!(engine.nn.layers[1].biases.len(), 8); // second hidden
        assert_eq!(engine.nn.layers[2].biases.len(), 3); // output: kp, ki, kd
    }
}
