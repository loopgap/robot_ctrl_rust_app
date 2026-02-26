use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PidController {
    pub kp: f64,
    pub ki: f64,
    pub kd: f64,
    pub setpoint: f64,
    pub output_limit: f64,
    pub integral_limit: f64,

    #[serde(skip)]
    pub integral: f64,
    #[serde(skip)]
    pub last_error: f64,
    #[serde(skip)]
    pub derivative: f64,
    #[serde(skip)]
    last_update: Option<Instant>,
    #[serde(skip)]
    pub output: f64,

    // 高级功能
    pub derivative_filter: f64, // 微分低通滤波系数 (0-1)
    pub anti_windup: bool,      // 抗积分饱和
    pub feedforward: f64,       // 前馈增益
    pub dead_zone: f64,         // 死区
}

impl Default for PidController {
    fn default() -> Self {
        Self {
            kp: 1.0,
            ki: 0.1,
            kd: 0.01,
            setpoint: 0.0,
            output_limit: 100.0,
            integral_limit: 100.0,
            integral: 0.0,
            last_error: 0.0,
            derivative: 0.0,
            last_update: None,
            output: 0.0,
            derivative_filter: 0.1,
            anti_windup: true,
            feedforward: 0.0,
            dead_zone: 0.0,
        }
    }
}

impl PidController {
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
        integral_limit: f64,
    ) -> Self {
        Self {
            kp,
            ki,
            kd,
            setpoint,
            output_limit,
            integral_limit,
            ..Default::default()
        }
    }

    pub fn compute(&mut self, feedback: f64) -> f64 {
        let now = Instant::now();
        let dt = match self.last_update {
            Some(last) => now.duration_since(last).as_secs_f64(),
            None => {
                self.last_update = Some(now);
                return 0.0;
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

        // P
        let p_term = self.kp * effective_error;

        // I (带抗饱和)
        self.integral += effective_error * dt;
        if self.anti_windup {
            self.integral = self
                .integral
                .clamp(-self.integral_limit, self.integral_limit);
        }
        let i_term = self.ki * self.integral;

        // D (带低通滤波)
        let raw_derivative = if dt > 0.0 {
            (effective_error - self.last_error) / dt
        } else {
            0.0
        };
        self.derivative = self.derivative * (1.0 - self.derivative_filter)
            + raw_derivative * self.derivative_filter;
        let d_term = self.kd * self.derivative;

        // 前馈
        let ff_term = self.feedforward * self.setpoint;

        let mut output = p_term + i_term + d_term + ff_term;
        output = output.clamp(-self.output_limit, self.output_limit);

        // 抗积分饱和：如果输出饱和，停止积分
        if self.anti_windup && output.abs() >= self.output_limit {
            self.integral -= effective_error * dt;
        }

        self.last_error = effective_error;
        self.last_update = Some(now);
        self.output = output;
        output
    }

    pub fn reset(&mut self) {
        self.integral = 0.0;
        self.last_error = 0.0;
        self.derivative = 0.0;
        self.last_update = None;
        self.output = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_pid_default() {
        let pid = PidController::default();
        assert_eq!(pid.kp, 1.0);
        assert_eq!(pid.ki, 0.1);
        assert_eq!(pid.kd, 0.01);
        assert_eq!(pid.setpoint, 0.0);
        assert_eq!(pid.output_limit, 100.0);
        assert_eq!(pid.integral_limit, 100.0);
        assert!(pid.anti_windup);
    }

    #[test]
    fn test_pid_new() {
        let pid = PidController::new(2.0, 0.5, 0.1, 10.0);
        assert_eq!(pid.kp, 2.0);
        assert_eq!(pid.ki, 0.5);
        assert_eq!(pid.kd, 0.1);
        assert_eq!(pid.setpoint, 10.0);
    }

    #[test]
    fn test_pid_with_limits() {
        let pid = PidController::with_limits(1.0, 0.1, 0.01, 5.0, 50.0, 25.0);
        assert_eq!(pid.output_limit, 50.0);
        assert_eq!(pid.integral_limit, 25.0);
        assert_eq!(pid.setpoint, 5.0);
    }

    #[test]
    fn test_pid_first_compute_returns_zero() {
        let mut pid = PidController::new(1.0, 0.0, 0.0, 100.0);
        // 第一次 compute 返回 0（因为需要两个时间点计算 dt）
        let out = pid.compute(0.0);
        assert_eq!(out, 0.0);
    }

    #[test]
    fn test_pid_compute_proportional() {
        let mut pid = PidController::new(2.0, 0.0, 0.0, 10.0);
        pid.dead_zone = 0.0;
        pid.feedforward = 0.0;
        // 初始化时间基线
        pid.compute(0.0);
        thread::sleep(Duration::from_millis(10));
        let out = pid.compute(0.0);
        // error = 10.0 - 0.0 = 10.0; P = 2.0 * 10.0 = 20.0
        assert!(
            out > 0.0,
            "P-only output should be positive for positive error"
        );
    }

    #[test]
    fn test_pid_output_limit_clamp() {
        let mut pid = PidController::with_limits(100.0, 0.0, 0.0, 100.0, 50.0, 50.0);
        pid.dead_zone = 0.0;
        pid.feedforward = 0.0;
        pid.compute(0.0);
        thread::sleep(Duration::from_millis(10));
        let out = pid.compute(0.0);
        // P = 100 * 100 = 10000, should clamp to 50
        assert!(
            (out - 50.0).abs() < 0.001,
            "Output should be clamped to 50.0, got {}",
            out
        );
    }

    #[test]
    fn test_pid_dead_zone() {
        let mut pid = PidController::new(1.0, 0.0, 0.0, 10.0);
        pid.dead_zone = 20.0; // 死区比误差大
        pid.feedforward = 0.0;
        pid.compute(0.0);
        thread::sleep(Duration::from_millis(10));
        let out = pid.compute(0.0);
        // error=10.0 < dead_zone=20.0, effective_error=0, output=0
        assert_eq!(out, 0.0, "Output should be 0 within dead zone");
    }

    #[test]
    fn test_pid_reset() {
        let mut pid = PidController::new(1.0, 0.5, 0.1, 10.0);
        pid.compute(0.0);
        thread::sleep(Duration::from_millis(10));
        pid.compute(5.0);
        pid.reset();
        assert_eq!(pid.integral, 0.0);
        assert_eq!(pid.last_error, 0.0);
        assert_eq!(pid.derivative, 0.0);
        assert_eq!(pid.output, 0.0);
        assert!(pid.last_update.is_none());
    }

    #[test]
    fn test_pid_negative_error() {
        let mut pid = PidController::new(1.0, 0.0, 0.0, -10.0);
        pid.dead_zone = 0.0;
        pid.feedforward = 0.0;
        pid.compute(0.0);
        thread::sleep(Duration::from_millis(10));
        let out = pid.compute(0.0);
        // error = -10 - 0 = -10, P = 1.0 * (-10) = -10
        assert!(out < 0.0, "Output should be negative for negative error");
    }
}
