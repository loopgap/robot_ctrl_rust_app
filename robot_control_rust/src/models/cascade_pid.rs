// ═══════════════════════════════════════════════════════════════
// 串级 PID 控制器 (Cascade PID)
// ═══════════════════════════════════════════════════════════════
//
// 双环结构：
// - 外环（主环）：位置控制，输出作为内环的设定值
// - 内环（从环）：速度控制，输出为最终执行机构控制量
//
// 外环通常慢速、内环快速，能有效抑制内环扰动。
// 常用于电机位置-速度双闭环控制。

use serde::{Deserialize, Serialize};
use std::time::Instant;

/// 单环 PID 内部状态（不含 Instant，用于串级内部）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct PidLoop {
    kp: f64,
    ki: f64,
    kd: f64,
    output_limit: f64,
    integral_limit: f64,
    integral: f64,
    last_error: f64,
    derivative: f64,
    output: f64,
    derivative_filter: f64,
    anti_windup: bool,
}

impl PidLoop {
    fn new(kp: f64, ki: f64, kd: f64, output_limit: f64) -> Self {
        Self {
            kp,
            ki,
            kd,
            output_limit,
            integral_limit: output_limit,
            integral: 0.0,
            last_error: 0.0,
            derivative: 0.0,
            output: 0.0,
            derivative_filter: 0.1,
            anti_windup: true,
        }
    }

    fn compute(&mut self, setpoint: f64, feedback: f64, dt: f64) -> f64 {
        let error = setpoint - feedback;

        // P
        let p_term = self.kp * error;

        // I
        self.integral += error * dt;
        if self.anti_windup {
            self.integral = self
                .integral
                .clamp(-self.integral_limit, self.integral_limit);
        }
        let i_term = self.ki * self.integral;

        // D
        let raw_deriv = if dt > 0.0 {
            (error - self.last_error) / dt
        } else {
            0.0
        };
        self.derivative =
            self.derivative * (1.0 - self.derivative_filter) + raw_deriv * self.derivative_filter;
        let d_term = self.kd * self.derivative;

        let mut output = p_term + i_term + d_term;
        output = output.clamp(-self.output_limit, self.output_limit);

        if self.anti_windup && output.abs() >= self.output_limit {
            self.integral -= error * dt;
        }

        self.last_error = error;
        self.output = output;
        output
    }

    fn reset(&mut self) {
        self.integral = 0.0;
        self.last_error = 0.0;
        self.derivative = 0.0;
        self.output = 0.0;
    }
}

/// 串级 PID 控制器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CascadePidController {
    // 外环参数（位置环）
    pub outer_kp: f64,
    pub outer_ki: f64,
    pub outer_kd: f64,
    pub outer_output_limit: f64,

    // 内环参数（速度环）
    pub inner_kp: f64,
    pub inner_ki: f64,
    pub inner_kd: f64,
    pub inner_output_limit: f64,

    // 全局设定值
    pub setpoint: f64,

    // 内部 PID 环
    #[serde(skip)]
    outer: PidLoop,
    #[serde(skip)]
    inner: PidLoop,

    #[serde(skip)]
    last_update: Option<Instant>,
    #[serde(skip)]
    pub output: f64,
    #[serde(skip)]
    pub outer_output: f64, // 外环输出（= 内环设定值）
    #[serde(skip)]
    pub inner_feedback: f64, // 内环反馈值（速度）
    #[serde(skip)]
    initialized: bool,
}

impl Default for CascadePidController {
    fn default() -> Self {
        let outer = PidLoop::new(1.0, 0.05, 0.02, 50.0);
        let inner = PidLoop::new(2.0, 0.2, 0.01, 100.0);
        Self {
            outer_kp: 1.0,
            outer_ki: 0.05,
            outer_kd: 0.02,
            outer_output_limit: 50.0,
            inner_kp: 2.0,
            inner_ki: 0.2,
            inner_kd: 0.01,
            inner_output_limit: 100.0,
            setpoint: 0.0,
            outer,
            inner,
            last_update: None,
            output: 0.0,
            outer_output: 0.0,
            inner_feedback: 0.0,
            initialized: false,
        }
    }
}

impl CascadePidController {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        outer_kp: f64,
        outer_ki: f64,
        outer_kd: f64,
        outer_limit: f64,
        inner_kp: f64,
        inner_ki: f64,
        inner_kd: f64,
        inner_limit: f64,
        setpoint: f64,
    ) -> Self {
        Self {
            outer_kp,
            outer_ki,
            outer_kd,
            outer_output_limit: outer_limit,
            inner_kp,
            inner_ki,
            inner_kd,
            inner_output_limit: inner_limit,
            setpoint,
            outer: PidLoop::new(outer_kp, outer_ki, outer_kd, outer_limit),
            inner: PidLoop::new(inner_kp, inner_ki, inner_kd, inner_limit),
            last_update: None,
            output: 0.0,
            outer_output: 0.0,
            inner_feedback: 0.0,
            initialized: false,
        }
    }

    /// 同步用户参数到内部 PID 环
    fn sync_params(&mut self) {
        self.outer.kp = self.outer_kp;
        self.outer.ki = self.outer_ki;
        self.outer.kd = self.outer_kd;
        self.outer.output_limit = self.outer_output_limit;
        self.outer.integral_limit = self.outer_output_limit;
        self.inner.kp = self.inner_kp;
        self.inner.ki = self.inner_ki;
        self.inner.kd = self.inner_kd;
        self.inner.output_limit = self.inner_output_limit;
        self.inner.integral_limit = self.inner_output_limit;
    }

    /// 计算串级控制输出
    ///
    /// - `position_feedback`: 位置反馈（外环）
    /// - `velocity_feedback`: 速度反馈（内环）
    pub fn compute(&mut self, position_feedback: f64, velocity_feedback: f64) -> f64 {
        let now = Instant::now();
        let dt = match self.last_update {
            Some(last) => now.duration_since(last).as_secs_f64(),
            None => {
                self.last_update = Some(now);
                self.initialized = true;
                return 0.0;
            }
        };
        if dt <= 0.0 {
            return self.output;
        }

        self.sync_params();

        // 外环：位置 → 速度设定值
        self.outer_output = self.outer.compute(self.setpoint, position_feedback, dt);

        // 内环：速度设定值 → 输出
        self.inner_feedback = velocity_feedback;
        self.output = self.inner.compute(self.outer_output, velocity_feedback, dt);

        self.last_update = Some(now);
        self.output
    }

    /// 单反馈简化版（用位置反馈数值差分估计速度）
    pub fn compute_single_feedback(&mut self, position_feedback: f64) -> f64 {
        let now = Instant::now();
        let dt = match self.last_update {
            Some(last) => now.duration_since(last).as_secs_f64(),
            None => {
                self.last_update = Some(now);
                self.inner_feedback = position_feedback;
                self.initialized = true;
                return 0.0;
            }
        };
        if dt <= 0.0 {
            return self.output;
        }

        self.sync_params();

        // 估计速度
        let velocity_est = (position_feedback - self.inner_feedback) / dt;

        // 外环
        self.outer_output = self.outer.compute(self.setpoint, position_feedback, dt);

        // 内环
        self.inner_feedback = position_feedback;
        self.output = self.inner.compute(self.outer_output, velocity_est, dt);

        self.last_update = Some(now);
        self.output
    }

    pub fn reset(&mut self) {
        self.outer.reset();
        self.inner.reset();
        self.last_update = None;
        self.output = 0.0;
        self.outer_output = 0.0;
        self.inner_feedback = 0.0;
        self.initialized = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_cascade_default() {
        let c = CascadePidController::default();
        assert_eq!(c.outer_kp, 1.0);
        assert_eq!(c.inner_kp, 2.0);
        assert_eq!(c.setpoint, 0.0);
        assert_eq!(c.output, 0.0);
    }

    #[test]
    fn test_cascade_new() {
        let c = CascadePidController::new(1.0, 0.05, 0.02, 50.0, 2.0, 0.2, 0.01, 100.0, 10.0);
        assert_eq!(c.setpoint, 10.0);
        assert_eq!(c.outer_kp, 1.0);
        assert_eq!(c.inner_kp, 2.0);
    }

    #[test]
    fn test_first_compute_returns_zero() {
        let mut c = CascadePidController {
            setpoint: 100.0,
            ..Default::default()
        };
        let out = c.compute(0.0, 0.0);
        assert_eq!(out, 0.0);
    }

    #[test]
    fn test_cascade_positive_output() {
        let mut c = CascadePidController::new(2.0, 0.0, 0.0, 100.0, 1.0, 0.0, 0.0, 200.0, 10.0);
        c.compute(0.0, 0.0);
        thread::sleep(Duration::from_millis(10));
        let out = c.compute(0.0, 0.0);
        assert!(
            out > 0.0,
            "Should output positive for positive position error, got {}",
            out
        );
    }

    #[test]
    fn test_cascade_single_feedback() {
        let mut c = CascadePidController::new(2.0, 0.0, 0.0, 100.0, 1.0, 0.0, 0.0, 200.0, 10.0);
        c.compute_single_feedback(0.0);
        thread::sleep(Duration::from_millis(10));
        let out = c.compute_single_feedback(0.0);
        assert!(out > 0.0, "Single feedback should work, got {}", out);
    }

    #[test]
    fn test_cascade_output_limit() {
        let mut c = CascadePidController::new(100.0, 0.0, 0.0, 50.0, 100.0, 0.0, 0.0, 30.0, 100.0);
        c.compute(0.0, 0.0);
        thread::sleep(Duration::from_millis(10));
        let out = c.compute(0.0, 0.0);
        assert!(
            out <= 30.0,
            "Output should be limited by inner loop limit 30, got {}",
            out
        );
    }

    #[test]
    fn test_cascade_reset() {
        let mut c = CascadePidController {
            setpoint: 10.0,
            ..Default::default()
        };
        c.compute(0.0, 0.0);
        thread::sleep(Duration::from_millis(10));
        c.compute(5.0, 1.0);
        c.reset();
        assert_eq!(c.output, 0.0);
        assert_eq!(c.outer_output, 0.0);
        assert!(c.last_update.is_none());
    }

    #[test]
    fn test_cascade_param_sync() {
        let mut c = CascadePidController {
            outer_kp: 5.0,
            inner_ki: 1.0,
            setpoint: 10.0,
            ..Default::default()
        };
        c.compute(0.0, 0.0);
        thread::sleep(Duration::from_millis(10));
        c.compute(0.0, 0.0);
        // 验证参数同步成功
        assert_eq!(c.outer.kp, 5.0);
        assert_eq!(c.inner.ki, 1.0);
    }

    #[test]
    fn test_cascade_inner_receives_outer_output() {
        let mut c = CascadePidController::new(2.0, 0.0, 0.0, 100.0, 1.0, 0.0, 0.0, 200.0, 10.0);
        c.compute(0.0, 0.0);
        thread::sleep(Duration::from_millis(10));
        c.compute(0.0, 0.0);
        // 外环有输出则内环有设定值
        assert!(
            c.outer_output.abs() > 0.0,
            "Outer should have non-zero output"
        );
    }
}
