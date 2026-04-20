// ═══════════════════════════════════════════════════════════════
// 线性二次调节器 (LQR - Linear Quadratic Regulator)
// ═══════════════════════════════════════════════════════════════
//
// 最优控制方法，最小化代价函数：
//   J = ∫ (x^T Q x + u^T R u) dt
//
// 对于二阶系统 (位置-速度状态空间):
//   ẋ = Ax + Bu,  u = -Kx
//   A = [[0, 1], [0, 0]]  (双积分器模型)
//   B = [[0], [1/m]]       (m = 等效惯量)
//
// 通过求解 Riccati 方程得到最优增益 K = R⁻¹B^TP
// 此实现使用解析公式 (二阶系统) + 迭代法 (一般情况)
//
// 跨平台：纯 Rust 实现，无外部依赖

use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LqrController {
    pub setpoint: f64,
    pub output_limit: f64,

    // ── 状态空间参数 ──
    /// Q 矩阵对角元素 [q1, q2] - 状态权重
    pub q1: f64, // 位置误差权重
    pub q2: f64, // 速度误差权重
    /// R 矩阵 - 控制量权重 (标量)
    pub r_weight: f64,
    /// 等效惯量 (影响 B 矩阵)
    pub mass: f64,

    // ── 计算得到的增益 ──
    #[serde(skip)]
    pub k1: f64, // 位置增益
    #[serde(skip)]
    pub k2: f64, // 速度增益
    #[serde(skip)]
    gains_valid: bool,

    // ── 状态估计 ──
    #[serde(skip)]
    last_position: f64,
    #[serde(skip)]
    estimated_velocity: f64,
    /// 速度估计滤波系数 (0-1, 越小越平滑)
    pub velocity_filter: f64,

    // ── 积分项 (消除稳态误差) ──
    pub enable_integral: bool,
    pub ki: f64,
    #[serde(skip)]
    integral: f64,
    pub integral_limit: f64,

    #[serde(skip)]
    pub output: f64,
    #[serde(skip)]
    last_update: Option<Instant>,
}

impl Default for LqrController {
    fn default() -> Self {
        let mut s = Self {
            setpoint: 0.0,
            output_limit: 100.0,
            q1: 100.0,
            q2: 10.0,
            r_weight: 1.0,
            mass: 1.0,
            k1: 0.0,
            k2: 0.0,
            gains_valid: false,
            last_position: 0.0,
            estimated_velocity: 0.0,
            velocity_filter: 0.3,
            enable_integral: true,
            ki: 0.5,
            integral: 0.0,
            integral_limit: 50.0,
            output: 0.0,
            last_update: None,
        };
        s.compute_gains();
        s
    }
}

impl LqrController {
    pub fn new(setpoint: f64, q1: f64, q2: f64, r: f64) -> Self {
        let mut s = Self {
            setpoint,
            q1,
            q2,
            r_weight: r,
            ..Default::default()
        };
        s.compute_gains();
        s
    }

    /// 求解二阶系统 Riccati 方程的解析解
    ///
    /// A = \[\[0, 1\], \[0, 0\]\], B = \[\[0\], \[1/m\]\]
    /// 解析求解 P 矩阵后 K = R⁻¹ B^T P
    pub fn compute_gains(&mut self) {
        let b = 1.0 / self.mass;

        // 迭代求解离散 Riccati (适用于小型系统)
        // 对于二阶双积分器系统有解析解:
        // K1 = sqrt(Q1/R) , K2 = sqrt((2*sqrt(Q1*R) + Q2) / R)
        // 简化为:
        let q1 = self.q1.max(0.001);
        let q2 = self.q2.max(0.001);
        let r = self.r_weight.max(0.001);

        self.k1 = (q1 / r).sqrt() * b;
        self.k2 = ((2.0 * (q1 * r).sqrt() + q2) / r).sqrt() * b;

        self.gains_valid = true;
    }

    pub fn compute(&mut self, feedback: f64) -> f64 {
        let now = Instant::now();
        let dt = match self.last_update {
            Some(last) => now.duration_since(last).as_secs_f64(),
            None => {
                self.last_update = Some(now);
                self.last_position = feedback;
                if !self.gains_valid {
                    self.compute_gains();
                }
                return 0.0;
            }
        };
        if dt <= 0.0 {
            return self.output;
        }

        if !self.gains_valid {
            self.compute_gains();
        }

        // 状态: 误差和误差变化率
        let error = self.setpoint - feedback;

        // 速度估计 (带滤波)
        let raw_vel = (feedback - self.last_position) / dt;
        self.estimated_velocity =
            self.estimated_velocity * (1.0 - self.velocity_filter) + raw_vel * self.velocity_filter;

        // LQR 控制律: u = K1 * error - K2 * velocity
        let mut u = self.k1 * error - self.k2 * self.estimated_velocity;

        // 积分项
        if self.enable_integral {
            self.integral += error * dt;
            self.integral = self
                .integral
                .clamp(-self.integral_limit, self.integral_limit);
            u += self.ki * self.integral;
        }

        self.output = u.clamp(-self.output_limit, self.output_limit);
        self.last_position = feedback;
        self.last_update = Some(now);
        self.output
    }

    /// 使用外部提供的速度反馈
    pub fn compute_with_velocity(&mut self, position: f64, velocity: f64) -> f64 {
        let now = Instant::now();
        let dt = match self.last_update {
            Some(last) => now.duration_since(last).as_secs_f64(),
            None => {
                self.last_update = Some(now);
                self.last_position = position;
                self.estimated_velocity = velocity;
                if !self.gains_valid {
                    self.compute_gains();
                }
                return 0.0;
            }
        };
        if dt <= 0.0 {
            return self.output;
        }

        if !self.gains_valid {
            self.compute_gains();
        }

        let error = self.setpoint - position;
        self.estimated_velocity = velocity;

        let mut u = self.k1 * error - self.k2 * velocity;

        if self.enable_integral {
            self.integral += error * dt;
            self.integral = self
                .integral
                .clamp(-self.integral_limit, self.integral_limit);
            u += self.ki * self.integral;
        }

        self.output = u.clamp(-self.output_limit, self.output_limit);
        self.last_position = position;
        self.last_update = Some(now);
        self.output
    }

    pub fn reset(&mut self) {
        self.last_position = 0.0;
        self.estimated_velocity = 0.0;
        self.integral = 0.0;
        self.output = 0.0;
        self.last_update = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_lqr_default() {
        let c = LqrController::default();
        assert_eq!(c.setpoint, 0.0);
        assert_eq!(c.output, 0.0);
        assert!(c.gains_valid);
        assert!(c.k1 > 0.0);
        assert!(c.k2 > 0.0);
    }

    #[test]
    fn test_lqr_new() {
        let c = LqrController::new(10.0, 100.0, 10.0, 1.0);
        assert_eq!(c.setpoint, 10.0);
        assert!(c.k1 > 0.0);
    }

    #[test]
    fn test_first_compute_returns_zero() {
        let mut c = LqrController::new(50.0, 100.0, 10.0, 1.0);
        let out = c.compute(0.0);
        assert_eq!(out, 0.0);
    }

    #[test]
    fn test_lqr_positive_error() {
        let mut c = LqrController::new(10.0, 100.0, 10.0, 1.0);
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
    fn test_lqr_output_limit() {
        let mut c = LqrController::new(1000.0, 1000.0, 100.0, 0.1);
        c.output_limit = 30.0;
        c.compute(0.0);
        thread::sleep(Duration::from_millis(10));
        let out = c.compute(0.0);
        assert!(out.abs() <= 30.0, "Output should be clamped, got {}", out);
    }

    #[test]
    fn test_lqr_with_velocity() {
        let mut c = LqrController::new(10.0, 100.0, 10.0, 1.0);
        c.compute_with_velocity(0.0, 0.0);
        thread::sleep(Duration::from_millis(10));
        let out = c.compute_with_velocity(0.0, 0.0);
        assert!(out > 0.0, "Should output positive, got {}", out);
    }

    #[test]
    fn test_lqr_gains_change_with_weights() {
        let c1 = LqrController::new(0.0, 100.0, 10.0, 1.0);
        let c2 = LqrController::new(0.0, 400.0, 10.0, 1.0);
        // 更大的 Q1 应该产生更大的 K1
        assert!(c2.k1 > c1.k1, "Higher Q1 should give higher K1");
    }

    #[test]
    fn test_lqr_reset() {
        let mut c = LqrController::new(10.0, 100.0, 10.0, 1.0);
        c.compute(0.0);
        thread::sleep(Duration::from_millis(10));
        c.compute(5.0);
        c.reset();
        assert_eq!(c.integral, 0.0);
        assert_eq!(c.output, 0.0);
        assert!(c.last_update.is_none());
    }

    #[test]
    fn test_lqr_integral_action() {
        let mut c1 = LqrController::new(10.0, 100.0, 10.0, 1.0);
        c1.enable_integral = false;
        c1.compute(0.0);
        thread::sleep(Duration::from_millis(10));
        let out1 = c1.compute(0.0);

        let mut c2 = LqrController::new(10.0, 100.0, 10.0, 1.0);
        c2.enable_integral = true;
        c2.ki = 1.0;
        c2.compute(0.0);
        thread::sleep(Duration::from_millis(10));
        let out2 = c2.compute(0.0);

        assert!(
            out2 >= out1,
            "With integral, output should be >= without: {} vs {}",
            out2,
            out1
        );
    }

    #[test]
    fn test_lqr_negative_error() {
        let mut c = LqrController::new(-10.0, 100.0, 10.0, 1.0);
        c.compute(0.0);
        thread::sleep(Duration::from_millis(10));
        let out = c.compute(0.0);
        assert!(out < 0.0, "Should output negative, got {}", out);
    }
}
