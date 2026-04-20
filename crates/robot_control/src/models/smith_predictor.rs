// ═══════════════════════════════════════════════════════════════
// Smith 预估控制器
// ═══════════════════════════════════════════════════════════════
//
// Smith 预估器用于补偿过程中的纯时滞（dead time）。
// 通过内部过程模型预测无时滞的系统响应，将 PID 控制器
// 从时滞的影响中解耦出来。
//
// 结构：PID + 过程模型（一阶 + 纯延迟）
// 适用于有明显传输/响应延迟的场景（如远程控制、液压系统）。
// 纯 Rust 实现，使用 VecDeque 模拟延迟缓冲，跨平台。

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::Instant;

/// Smith 预估控制器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmithPredictorController {
    // PID 参数
    pub kp: f64,
    pub ki: f64,
    pub kd: f64,
    pub setpoint: f64,
    pub output_limit: f64,
    pub integral_limit: f64,

    // 过程模型参数
    pub model_gain: f64,       // 过程增益 K
    pub model_time_const: f64, // 一阶时间常数 T (秒)
    pub model_dead_time: f64,  // 纯时滞 L (秒)

    // PID 内部状态
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

    // 模型内部状态
    #[serde(skip)]
    model_state: f64, // 无延迟模型输出
    #[serde(skip)]
    delay_buffer: VecDeque<f64>, // 延迟缓冲区
    #[serde(skip)]
    model_delayed_output: f64, // 延迟后的模型输出

    pub anti_windup: bool,
    pub derivative_filter: f64,
}

impl Default for SmithPredictorController {
    fn default() -> Self {
        Self {
            kp: 1.0,
            ki: 0.1,
            kd: 0.01,
            setpoint: 0.0,
            output_limit: 100.0,
            integral_limit: 100.0,
            model_gain: 1.0,
            model_time_const: 0.5,
            model_dead_time: 0.1,
            integral: 0.0,
            last_error: 0.0,
            derivative: 0.0,
            last_update: None,
            output: 0.0,
            model_state: 0.0,
            delay_buffer: VecDeque::new(),
            model_delayed_output: 0.0,
            anti_windup: true,
            derivative_filter: 0.1,
        }
    }
}

impl SmithPredictorController {
    pub fn new(kp: f64, ki: f64, kd: f64, setpoint: f64) -> Self {
        Self {
            kp,
            ki,
            kd,
            setpoint,
            ..Default::default()
        }
    }

    pub fn with_model(
        kp: f64,
        ki: f64,
        kd: f64,
        setpoint: f64,
        gain: f64,
        time_const: f64,
        dead_time: f64,
    ) -> Self {
        Self {
            kp,
            ki,
            kd,
            setpoint,
            model_gain: gain,
            model_time_const: time_const,
            model_dead_time: dead_time,
            ..Default::default()
        }
    }

    /// 更新一阶过程模型（无延迟部分）
    ///
    /// G(s) = K / (Ts + 1)
    /// 离散化：y(k) = y(k-1) + dt/T * (K*u - y(k-1))
    fn update_model(&mut self, control_input: f64, dt: f64) {
        if self.model_time_const > 0.0 {
            let alpha = dt / self.model_time_const;
            self.model_state += alpha * (self.model_gain * control_input - self.model_state);
        } else {
            self.model_state = self.model_gain * control_input;
        }

        // 将当前无延迟模型输出推入延迟缓冲
        self.delay_buffer.push_back(self.model_state);

        // 根据 dead_time 和 dt 计算缓冲区应保持的长度
        let buffer_len = if dt > 0.0 {
            (self.model_dead_time / dt).ceil() as usize
        } else {
            1
        };
        let buffer_len = buffer_len.max(1);

        // 取出延迟后的模型输出
        if self.delay_buffer.len() > buffer_len {
            self.model_delayed_output = self.delay_buffer.pop_front().unwrap_or(0.0);
        } else if !self.delay_buffer.is_empty() {
            self.model_delayed_output = self.delay_buffer[0];
        }
    }

    /// 计算 Smith 预估控制输出
    ///
    /// 补偿信号 = 无延迟模型输出 - 延迟模型输出
    /// PID 输入误差 = setpoint - (feedback + 补偿信号)  ... 实际上是
    /// PID 输入 = setpoint - feedback - (model_no_delay - model_delayed)
    /// 等效于 PID 看到的是无时滞过程的响应
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

        // Smith 补偿：将模型预测的时滞效应从反馈中去除
        let compensation = self.model_state - self.model_delayed_output;
        let compensated_feedback = feedback + compensation;

        let error = self.setpoint - compensated_feedback;

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

        let mut control = p_term + i_term + d_term;
        control = control.clamp(-self.output_limit, self.output_limit);

        // 抗积分饱和
        if self.anti_windup && control.abs() >= self.output_limit {
            self.integral -= error * dt;
        }

        // 更新过程模型
        self.update_model(control, dt);

        self.last_error = error;
        self.last_update = Some(now);
        self.output = control;
        control
    }

    pub fn reset(&mut self) {
        self.integral = 0.0;
        self.last_error = 0.0;
        self.derivative = 0.0;
        self.last_update = None;
        self.output = 0.0;
        self.model_state = 0.0;
        self.delay_buffer.clear();
        self.model_delayed_output = 0.0;
    }

    /// 获取当前模型预测状态（用于 UI 显示）
    pub fn model_prediction(&self) -> f64 {
        self.model_state
    }

    /// 获取延迟缓冲区长度（用于 UI 显示）
    pub fn delay_buffer_len(&self) -> usize {
        self.delay_buffer.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_smith_default() {
        let c = SmithPredictorController::default();
        assert_eq!(c.kp, 1.0);
        assert_eq!(c.model_gain, 1.0);
        assert_eq!(c.model_time_const, 0.5);
        assert_eq!(c.model_dead_time, 0.1);
        assert_eq!(c.output, 0.0);
    }

    #[test]
    fn test_smith_new() {
        let c = SmithPredictorController::new(2.0, 0.5, 0.1, 10.0);
        assert_eq!(c.kp, 2.0);
        assert_eq!(c.setpoint, 10.0);
    }

    #[test]
    fn test_smith_with_model() {
        let c = SmithPredictorController::with_model(1.0, 0.1, 0.01, 5.0, 2.0, 1.0, 0.5);
        assert_eq!(c.model_gain, 2.0);
        assert_eq!(c.model_time_const, 1.0);
        assert_eq!(c.model_dead_time, 0.5);
    }

    #[test]
    fn test_first_compute_returns_zero() {
        let mut c = SmithPredictorController::new(1.0, 0.0, 0.0, 100.0);
        let out = c.compute(0.0);
        assert_eq!(out, 0.0);
    }

    #[test]
    fn test_smith_positive_output() {
        let mut c = SmithPredictorController::new(2.0, 0.0, 0.0, 10.0);
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
    fn test_smith_output_limit() {
        let mut c = SmithPredictorController::new(100.0, 0.0, 0.0, 100.0);
        c.output_limit = 50.0;
        c.compute(0.0);
        thread::sleep(Duration::from_millis(10));
        let out = c.compute(0.0);
        assert!(out <= 50.0, "Output should be clamped to 50.0, got {}", out);
    }

    #[test]
    fn test_smith_model_updates() {
        let mut c = SmithPredictorController::with_model(1.0, 0.0, 0.0, 10.0, 1.0, 0.5, 0.1);
        c.compute(0.0);
        thread::sleep(Duration::from_millis(20));
        c.compute(0.0);
        // 模型应该有响应
        assert!(
            c.model_state.abs() > 0.0,
            "Model state should change after compute"
        );
    }

    #[test]
    fn test_smith_delay_buffer() {
        let mut c = SmithPredictorController::with_model(1.0, 0.0, 0.0, 10.0, 1.0, 0.1, 0.5);
        c.compute(0.0);
        for _ in 0..10 {
            thread::sleep(Duration::from_millis(10));
            c.compute(0.0);
        }
        assert!(c.delay_buffer_len() > 0, "Delay buffer should have entries");
    }

    #[test]
    fn test_smith_reset() {
        let mut c = SmithPredictorController::new(1.0, 0.5, 0.1, 10.0);
        c.compute(0.0);
        thread::sleep(Duration::from_millis(10));
        c.compute(5.0);
        c.reset();
        assert_eq!(c.integral, 0.0);
        assert_eq!(c.last_error, 0.0);
        assert_eq!(c.output, 0.0);
        assert_eq!(c.model_state, 0.0);
        assert!(c.delay_buffer.is_empty());
        assert!(c.last_update.is_none());
    }

    #[test]
    fn test_smith_negative_error() {
        let mut c = SmithPredictorController::new(1.0, 0.0, 0.0, -10.0);
        c.compute(0.0);
        thread::sleep(Duration::from_millis(10));
        let out = c.compute(0.0);
        assert!(
            out < 0.0,
            "Should output negative for negative setpoint, got {}",
            out
        );
    }

    #[test]
    fn test_model_prediction() {
        let mut c = SmithPredictorController::with_model(1.0, 0.0, 0.0, 10.0, 2.0, 0.1, 0.05);
        c.compute(0.0);
        thread::sleep(Duration::from_millis(10));
        c.compute(0.0);
        let pred = c.model_prediction();
        // 模型应已开始预测
        assert!(
            pred != 0.0 || c.delay_buffer_len() > 0,
            "Model should have prediction data"
        );
    }
}
