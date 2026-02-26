// ═══════════════════════════════════════════════════════════════
// 模型预测控制器 (MPC - Model Predictive Control)
// ═══════════════════════════════════════════════════════════════
//
// 在每个控制周期内：
// 1. 用内部模型预测未来 N 步的系统行为
// 2. 在线求解优化问题，使预测轨迹接近目标
// 3. 仅执行第一步控制量，下周期重新预测
//
// 本实现采用二阶离散状态空间模型 + QP 简化求解：
//   x(k+1) = A_d x(k) + B_d u(k)
//   J = Σ (x^T Q x + u^T R u + Δu^T S Δu)
//
// 支持：控制量约束、变化率约束、预测/控制步长配置
// 跨平台：纯 Rust 实现

use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MpcController {
    pub setpoint: f64,
    pub output_limit: f64,

    // ── MPC 参数 ──
    /// 预测步长 Np
    pub prediction_horizon: usize,
    /// 控制步长 Nc (Nc <= Np)
    pub control_horizon: usize,
    /// 状态权重 Q (位置误差)
    pub q_weight: f64,
    /// 控制量权重 R
    pub r_weight: f64,
    /// 控制变化率权重 S
    pub s_weight: f64,

    // ── 内部过程模型 ──
    /// 模型增益
    pub model_gain: f64,
    /// 模型时间常数
    pub model_time_const: f64,
    /// 采样周期 (秒)
    pub sample_time: f64,

    /// 控制量变化率约束 |Δu| <= du_limit
    pub du_limit: f64,

    // ── 内部状态 ──
    #[serde(skip)]
    pub model_state: f64, // 内部模型状态
    #[serde(skip)]
    pub last_u: f64, // 上一步控制量
    #[serde(skip)]
    pub predicted_output: f64, // 最近预测输出
    #[serde(skip)]
    pub output: f64,
    #[serde(skip)]
    last_update: Option<Instant>,
    #[serde(skip)]
    accumulated_time: f64,
}

impl Default for MpcController {
    fn default() -> Self {
        Self {
            setpoint: 0.0,
            output_limit: 100.0,
            prediction_horizon: 10,
            control_horizon: 3,
            q_weight: 100.0,
            r_weight: 1.0,
            s_weight: 10.0,
            model_gain: 1.0,
            model_time_const: 0.5,
            sample_time: 0.05,
            du_limit: 20.0,
            model_state: 0.0,
            last_u: 0.0,
            predicted_output: 0.0,
            output: 0.0,
            last_update: None,
            accumulated_time: 0.0,
        }
    }
}

impl MpcController {
    pub fn new(setpoint: f64, np: usize, nc: usize) -> Self {
        Self {
            setpoint,
            prediction_horizon: np.max(2),
            control_horizon: nc.max(1).min(np.max(2)),
            ..Default::default()
        }
    }

    /// 离散化一阶模型参数
    /// G(s) = K / (Ts + 1)  →  y(k+1) = a * y(k) + b * u(k)
    fn discrete_params(&self) -> (f64, f64) {
        let ts = self.sample_time.max(0.001);
        let tc = self.model_time_const.max(0.001);
        let a = (-ts / tc).exp();
        let b = self.model_gain * (1.0 - a);
        (a, b)
    }

    /// 简化 MPC 求解：梯度下降法求最优 Δu 序列
    fn solve_mpc(&self, current_state: f64) -> f64 {
        let nc = self.control_horizon.min(self.prediction_horizon).max(1);
        let np = self.prediction_horizon.max(2);
        let (a, b) = self.discrete_params();

        // 初始化 Δu 序列为 0
        let mut du_seq = vec![0.0_f64; nc];

        // 自适应学习率梯度下降求解优化问题
        let base_lr = 0.01;
        let iterations = 100;

        let mut best_cost = self.evaluate_cost(&du_seq, current_state, a, b, np);
        let mut best_du = du_seq.clone();

        for iter in 0..iterations {
            // 计算代价和梯度
            let mut gradients = vec![0.0_f64; nc];

            for j in 0..nc {
                let eps = 0.01;

                // f(du + eps)
                du_seq[j] += eps;
                let cost_plus = self.evaluate_cost(&du_seq, current_state, a, b, np);

                // f(du - eps)
                du_seq[j] -= 2.0 * eps;
                let cost_minus = self.evaluate_cost(&du_seq, current_state, a, b, np);

                du_seq[j] += eps; // 恢复
                gradients[j] = (cost_plus - cost_minus) / (2.0 * eps);
            }

            // 自适应学习率：随迭代衰减
            let lr = base_lr / (1.0 + 0.01 * iter as f64);

            // 更新
            for j in 0..nc {
                du_seq[j] -= lr * gradients[j];
                du_seq[j] = du_seq[j].clamp(-self.du_limit, self.du_limit);
            }

            // 保留最优解
            let cost = self.evaluate_cost(&du_seq, current_state, a, b, np);
            if cost < best_cost {
                best_cost = cost;
                best_du.clone_from(&du_seq);
            }
        }

        best_du[0]
    }

    /// 评估代价函数
    #[allow(clippy::needless_range_loop)]
    fn evaluate_cost(&self, du_seq: &[f64], initial_state: f64, a: f64, b: f64, np: usize) -> f64 {
        let nc = du_seq.len();
        let mut cost = 0.0;
        let mut state = initial_state;
        let mut u = self.last_u;

        for k in 0..np {
            let du = if k < nc { du_seq[k] } else { 0.0 };
            u += du;
            u = u.clamp(-self.output_limit, self.output_limit);

            state = a * state + b * u;

            let error = self.setpoint - state;
            cost += self.q_weight * error * error;
            cost += self.r_weight * u * u;
            cost += self.s_weight * du * du;
        }

        cost
    }

    pub fn compute(&mut self, feedback: f64) -> f64 {
        let now = Instant::now();
        let dt = match self.last_update {
            Some(last) => now.duration_since(last).as_secs_f64(),
            None => {
                self.last_update = Some(now);
                self.model_state = feedback;
                return 0.0;
            }
        };
        if dt <= 0.0 {
            return self.output;
        }

        self.accumulated_time += dt;

        // 仅在采样时间到达时计算
        if self.accumulated_time >= self.sample_time {
            self.accumulated_time = 0.0;

            // 用实际反馈修正模型状态
            self.model_state = feedback;

            // 求解 MPC
            let du = self.solve_mpc(self.model_state);
            self.last_u += du;
            self.last_u = self.last_u.clamp(-self.output_limit, self.output_limit);

            // 更新模型预测
            let (a, b) = self.discrete_params();
            self.predicted_output = a * self.model_state + b * self.last_u;
        }

        self.output = self.last_u;
        self.last_update = Some(now);
        self.output
    }

    pub fn reset(&mut self) {
        self.model_state = 0.0;
        self.last_u = 0.0;
        self.predicted_output = 0.0;
        self.output = 0.0;
        self.accumulated_time = 0.0;
        self.last_update = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_mpc_default() {
        let c = MpcController::default();
        assert_eq!(c.setpoint, 0.0);
        assert_eq!(c.output, 0.0);
        assert_eq!(c.prediction_horizon, 10);
        assert_eq!(c.control_horizon, 3);
    }

    #[test]
    fn test_mpc_new() {
        let c = MpcController::new(10.0, 20, 5);
        assert_eq!(c.setpoint, 10.0);
        assert_eq!(c.prediction_horizon, 20);
        assert_eq!(c.control_horizon, 5);
    }

    #[test]
    fn test_first_compute_returns_zero() {
        let mut c = MpcController::new(50.0, 10, 3);
        let out = c.compute(0.0);
        assert_eq!(out, 0.0);
    }

    #[test]
    fn test_mpc_positive_error() {
        let mut c = MpcController::new(10.0, 10, 3);
        c.sample_time = 0.001; // 快速采样以便测试
        c.compute(0.0);
        thread::sleep(Duration::from_millis(10));
        let out = c.compute(0.0);
        assert!(
            out > 0.0,
            "Should output positive for positive error, got {}",
            out
        );
    }

    #[test]
    fn test_mpc_output_limit() {
        let mut c = MpcController::new(1000.0, 10, 3);
        c.output_limit = 30.0;
        c.sample_time = 0.001;
        c.compute(0.0);
        thread::sleep(Duration::from_millis(10));
        let out = c.compute(0.0);
        assert!(out.abs() <= 30.0, "Output should be clamped, got {}", out);
    }

    #[test]
    fn test_mpc_reset() {
        let mut c = MpcController::new(10.0, 10, 3);
        c.sample_time = 0.001;
        c.compute(0.0);
        thread::sleep(Duration::from_millis(10));
        c.compute(5.0);
        c.reset();
        assert_eq!(c.model_state, 0.0);
        assert_eq!(c.last_u, 0.0);
        assert_eq!(c.output, 0.0);
        assert!(c.last_update.is_none());
    }

    #[test]
    fn test_mpc_discrete_params() {
        let c = MpcController::default();
        let (a, b) = c.discrete_params();
        assert!(
            a > 0.0 && a < 1.0,
            "Discrete a should be in (0,1), got {}",
            a
        );
        assert!(b > 0.0, "Discrete b should be positive, got {}", b);
    }

    #[test]
    fn test_mpc_negative_error() {
        let mut c = MpcController::new(-10.0, 10, 3);
        c.sample_time = 0.001;
        c.compute(0.0);
        thread::sleep(Duration::from_millis(10));
        let out = c.compute(0.0);
        assert!(out < 0.0, "Should output negative, got {}", out);
    }

    #[test]
    fn test_mpc_control_horizon_clamped() {
        let c = MpcController::new(0.0, 5, 10);
        assert!(c.control_horizon <= c.prediction_horizon);
    }

    #[test]
    fn test_evaluate_cost_finite() {
        let c = MpcController::default();
        let du = vec![1.0, 0.5, 0.0];
        let (a, b) = c.discrete_params();
        let cost = c.evaluate_cost(&du, 0.0, a, b, 10);
        assert!(cost.is_finite());
        assert!(cost >= 0.0);
    }
}
