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
    fn test_new_creates_10_algorithms() {
        let engine = ControlEngine::new();
        assert_eq!(engine.algorithms.len(), 10);
    }

    #[test]
    fn test_active_algorithm_name() {
        let engine = ControlEngine::new();
        assert_eq!(engine.algorithm_name(), "Classic PID");
    }

    #[test]
    fn test_set_active_algorithm() {
        let mut engine = ControlEngine::new();
        engine.set_active_algorithm(6);
        assert_eq!(engine.algorithm_name(), "ADRC");
    }

    #[test]
    fn test_set_active_algorithm_out_of_range_ignored() {
        let mut engine = ControlEngine::new();
        engine.set_active_algorithm(999);
        assert_eq!(engine.active_index, 0);
    }

    #[test]
    fn test_compute_returns_finite() {
        let mut engine = ControlEngine::new();
        assert!(engine.compute(1.0).is_finite());
    }

    #[test]
    fn test_setpoint_default() {
        let engine = ControlEngine::new();
        assert_eq!(engine.setpoint(), 0.0);
    }

    #[test]
    fn test_set_setpoint() {
        let mut engine = ControlEngine::new();
        engine.set_setpoint(42.0);
        assert_eq!(engine.setpoint(), 42.0);
    }

    #[test]
    fn test_output_after_compute() {
        let mut engine = ControlEngine::new();
        engine.compute(1.0);
        assert_eq!(engine.output(), engine.active_algorithm().output());
    }

    #[test]
    fn test_toggle_running() {
        let mut engine = ControlEngine::new();
        assert!(!engine.is_running);
        engine.toggle_running();
        assert!(engine.is_running);
        engine.toggle_running();
        assert!(!engine.is_running);
    }

    #[test]
    fn test_emergency_stop() {
        let mut engine = ControlEngine::new();
        engine.is_running = true;
        engine.emergency_stop();
        assert!(!engine.is_running);
        assert_eq!(engine.setpoint(), 0.0);
    }

    #[test]
    fn test_reset_active() {
        let mut engine = ControlEngine::new();
        engine.compute(1.0);
        engine.reset_active();
        assert_eq!(engine.output(), 0.0);
    }

    #[test]
    fn test_push_state_limits_history() {
        let mut engine = ControlEngine::new();
        for i in 0..10001 {
            engine.push_state(RobotState {
                position: i as f64,
                ..Default::default()
            });
        }
        assert_eq!(engine.state_history.len(), 10000);
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
            assert_eq!(engine.output(), 0.0);
        }
    }

    #[test]
    fn test_compute_dual_for_cascade_pid() {
        let mut engine = ControlEngine::new();
        engine.set_active_algorithm(4);
        assert!(engine.compute_dual(1.0, 0.5).is_finite());
    }

    #[test]
    fn test_compute_dual_for_lqr() {
        let mut engine = ControlEngine::new();
        engine.set_active_algorithm(8);
        assert!(engine.compute_dual(1.0, 0.5).is_finite());
    }

    #[test]
    fn test_topology_defaults() {
        let engine = ControlEngine::new();
        assert!(!engine.builtin_topologies.is_empty());
    }

    #[test]
    fn test_dual_feedback_control_trait_cascade_pid() {
        let mut engine = ControlEngine::new();
        engine.set_active_algorithm(4);

        let algorithm = engine.active_algorithm_mut();
        if let Some(dual) = algorithm
            .as_any_mut()
            .downcast_mut::<CascadePidController>()
        {
            let output = dual.compute_dual(10.0, 1.0);
            assert!(output.is_finite());
        } else {
            panic!("CascadePidController should implement DualFeedbackControl");
        }
    }

    #[test]
    fn test_dual_feedback_control_trait_lqr() {
        let mut engine = ControlEngine::new();
        engine.set_active_algorithm(8);

        let algorithm = engine.active_algorithm_mut();
        if let Some(dual) = algorithm.as_any_mut().downcast_mut::<LqrController>() {
            let output = dual.compute_dual(10.0, 1.0);
            assert!(output.is_finite());
        } else {
            panic!("LqrController should implement DualFeedbackControl");
        }
    }

    #[test]
    fn test_all_algorithms_have_names() {
        for i in 0..10 {
            let mut e = ControlEngine::new();
            e.set_active_algorithm(i);
            assert!(!e.algorithm_name().is_empty());
        }
    }

    #[test]
    fn test_all_algorithms_reset_to_zero_v2() {
        for i in 0..10 {
            let mut e = ControlEngine::new();
            e.set_active_algorithm(i);
            e.compute(1.0);
            e.reset_active();
            assert_eq!(e.output(), 0.0);
        }
    }

    #[test]
    fn test_setpoint_roundtrip() {
        let mut engine = ControlEngine::new();
        engine.set_setpoint(42.0);
        assert_eq!(engine.setpoint(), 42.0);
    }

    #[test]
    fn test_emergency_stop_v2() {
        let mut engine = ControlEngine::new();
        engine.toggle_running();
        assert!(engine.is_running);
        engine.emergency_stop();
        assert!(!engine.is_running);
        assert_eq!(engine.output(), 0.0);
    }

    #[test]
    fn test_push_state_limits_history_v2() {
        let mut engine = ControlEngine::new();
        for i in 0..10001 {
            engine.push_state(RobotState {
                position: i as f64,
                ..Default::default()
            });
        }
        assert!(engine.state_history.len() <= 10000);
    }

    #[test]
    fn test_active_index_out_of_range_ignored() {
        let mut engine = ControlEngine::new();
        engine.set_active_algorithm(0);
        engine.set_active_algorithm(100);
        assert_eq!(engine.active_index, 0);
    }
}
