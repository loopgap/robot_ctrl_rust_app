// ═══════════════════════════════════════════════════════════════
// 增量式 PID 控制器
// ═══════════════════════════════════════════════════════════════
//
// 与位置式 PID 不同，增量式 PID 输出的是控制量的增量 Δu，
// 最终控制量 u(k) = u(k-1) + Δu(k)
// 优点：无积分饱和问题、支持无扰切换、计算量小

use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncrementalPidController {
    pub kp: f64,
    pub ki: f64,
    pub kd: f64,
    pub setpoint: f64,
    pub output_limit: f64,
    pub increment_limit: f64, // 单步增量限制

    #[serde(skip)]
    pub last_error: f64,
    #[serde(skip)]
    pub prev_error: f64, // e(k-2)
    #[serde(skip)]
    last_update: Option<Instant>,
    #[serde(skip)]
    pub output: f64,
    #[serde(skip)]
    pub last_increment: f64,

    // 高级功能
    pub dead_zone: f64,
    pub output_ramp: f64, // 输出斜率限制 (每秒最大变化量, 0=不限制)
}

impl Default for IncrementalPidController {
    fn default() -> Self {
        Self {
            kp: 1.0,
            ki: 0.1,
            kd: 0.01,
            setpoint: 0.0,
            output_limit: 100.0,
            increment_limit: 50.0,
            last_error: 0.0,
            prev_error: 0.0,
            last_update: None,
            output: 0.0,
            last_increment: 0.0,
            dead_zone: 0.0,
            output_ramp: 0.0,
        }
    }
}

impl IncrementalPidController {
    pub fn new(kp: f64, ki: f64, kd: f64, setpoint: f64) -> Self {
        Self {
            kp,
            ki,
            kd,
            setpoint,
            ..Default::default()
        }
    }

    pub fn with_limits(
        kp: f64,
        ki: f64,
        kd: f64,
        setpoint: f64,
        output_limit: f64,
        increment_limit: f64,
    ) -> Self {
        Self {
            kp,
            ki,
            kd,
            setpoint,
            output_limit,
            increment_limit,
            ..Default::default()
        }
    }

    /// 计算控制增量 Δu 并累加到输出
    ///
    /// Δu(k) = Kp * [e(k) - e(k-1)] + Ki * e(k) + Kd * [e(k) - 2*e(k-1) + e(k-2)]
    pub fn compute(&mut self, feedback: f64) -> f64 {
        let now = Instant::now();
        let dt = match self.last_update {
            Some(last) => now.duration_since(last).as_secs_f64(),
            None => {
                self.last_update = Some(now);
                let raw_error = self.setpoint - feedback;
                self.last_error = if raw_error.abs() < self.dead_zone {
                    0.0
                } else {
                    raw_error
                };
                return self.output;
            }
        };
        if dt <= 0.0 {
            return self.output;
        }

        let error = self.setpoint - feedback;

        // 死区处理
        let effective_error = if error.abs() < self.dead_zone {
            0.0
        } else {
            error
        };

        // 增量式公式
        let delta_p = self.kp * (effective_error - self.last_error);
        let delta_i = self.ki * effective_error * dt;
        let delta_d = if dt > 0.0 {
            self.kd * (effective_error - 2.0 * self.last_error + self.prev_error) / dt
        } else {
            0.0
        };

        let mut increment = delta_p + delta_i + delta_d;

        // 增量限幅
        increment = increment.clamp(-self.increment_limit, self.increment_limit);

        // 斜率限制
        if self.output_ramp > 0.0 {
            let max_change = self.output_ramp * dt;
            increment = increment.clamp(-max_change, max_change);
        }

        self.last_increment = increment;

        // 累加输出
        self.output += increment;
        self.output = self.output.clamp(-self.output_limit, self.output_limit);

        // 更新误差历史
        self.prev_error = self.last_error;
        self.last_error = effective_error;
        self.last_update = Some(now);

        self.output
    }

    pub fn reset(&mut self) {
        self.last_error = 0.0;
        self.prev_error = 0.0;
        self.last_update = None;
        self.output = 0.0;
        self.last_increment = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_incremental_pid_default() {
        let c = IncrementalPidController::default();
        assert_eq!(c.kp, 1.0);
        assert_eq!(c.ki, 0.1);
        assert_eq!(c.kd, 0.01);
        assert_eq!(c.output, 0.0);
        assert_eq!(c.increment_limit, 50.0);
    }

    #[test]
    fn test_incremental_pid_new() {
        let c = IncrementalPidController::new(2.0, 0.5, 0.1, 10.0);
        assert_eq!(c.kp, 2.0);
        assert_eq!(c.setpoint, 10.0);
    }

    #[test]
    fn test_incremental_pid_with_limits() {
        let c = IncrementalPidController::with_limits(1.0, 0.1, 0.01, 5.0, 50.0, 25.0);
        assert_eq!(c.output_limit, 50.0);
        assert_eq!(c.increment_limit, 25.0);
    }

    #[test]
    fn test_first_compute_returns_zero() {
        let mut c = IncrementalPidController::new(1.0, 0.0, 0.0, 100.0);
        let out = c.compute(0.0);
        assert_eq!(out, 0.0);
    }

    #[test]
    fn test_incremental_positive_direction() {
        let mut c = IncrementalPidController::new(2.0, 0.5, 0.0, 10.0);
        c.compute(0.0); // 初始化
        thread::sleep(Duration::from_millis(10));
        let out = c.compute(0.0);
        // 积分项 ki*e*dt > 0，输出应为正
        assert!(
            out > 0.0,
            "Output should be positive for positive error, got {}",
            out
        );
    }

    #[test]
    fn test_incremental_accumulation() {
        let mut c = IncrementalPidController::new(1.0, 1.0, 0.0, 10.0);
        c.compute(0.0);
        thread::sleep(Duration::from_millis(10));
        let out1 = c.compute(0.0);
        thread::sleep(Duration::from_millis(10));
        let out2 = c.compute(0.0);
        // 输出应该随着积分项持续增加
        assert!(
            out2 >= out1,
            "Output should accumulate: {} >= {}",
            out2,
            out1
        );
    }

    #[test]
    fn test_incremental_output_limit() {
        let mut c = IncrementalPidController::with_limits(100.0, 100.0, 0.0, 100.0, 50.0, 100.0);
        c.compute(0.0);
        for _ in 0..50 {
            thread::sleep(Duration::from_millis(2));
            c.compute(0.0);
        }
        assert!(
            c.output <= 50.0,
            "Output should be clamped to 50.0, got {}",
            c.output
        );
        assert!(c.output >= -50.0);
    }

    #[test]
    fn test_incremental_dead_zone() {
        let mut c = IncrementalPidController::new(1.0, 0.0, 0.0, 5.0);
        c.dead_zone = 10.0;
        c.compute(0.0);
        thread::sleep(Duration::from_millis(10));
        let out = c.compute(0.0);
        assert_eq!(out, 0.0, "Output should be 0 within dead zone");
    }

    #[test]
    fn test_incremental_reset() {
        let mut c = IncrementalPidController::new(1.0, 0.5, 0.1, 10.0);
        c.compute(0.0);
        thread::sleep(Duration::from_millis(10));
        c.compute(5.0);
        c.reset();
        assert_eq!(c.last_error, 0.0);
        assert_eq!(c.prev_error, 0.0);
        assert_eq!(c.output, 0.0);
        assert_eq!(c.last_increment, 0.0);
        assert!(c.last_update.is_none());
    }

    #[test]
    fn test_increment_limit() {
        let mut c = IncrementalPidController::with_limits(100.0, 0.0, 0.0, 100.0, 1000.0, 5.0);
        c.compute(0.0);
        thread::sleep(Duration::from_millis(10));
        c.compute(0.0);
        // 增量应被限制在 ±5
        assert!(
            c.last_increment.abs() <= 5.0 + 0.001,
            "Increment should be clamped to 5.0, got {}",
            c.last_increment
        );
    }
}
