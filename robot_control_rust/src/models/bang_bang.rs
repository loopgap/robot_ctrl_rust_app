// ═══════════════════════════════════════════════════════════════
// Bang-Bang (开关) 控制器
// ═══════════════════════════════════════════════════════════════
//
// 最简单的控制策略：误差 > 上阈值时输出正向满幅，
// 误差 < 下阈值时输出负向满幅，在回滞区内保持上次输出。
// 适合温控、简单位置控制等对精度要求不高的场景。

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BangBangController {
    pub setpoint: f64,
    pub output_high: f64, // 正向输出值
    pub output_low: f64,  // 负向输出值（通常为负）
    pub hysteresis: f64,  // 回滞区半宽（防抖动）
    pub dead_band: f64,   // 死区（误差在此范围内输出为0）

    #[serde(skip)]
    pub output: f64,
    #[serde(skip)]
    pub last_state: BangBangState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum BangBangState {
    #[default]
    Off,
    High,
    Low,
}

impl Default for BangBangController {
    fn default() -> Self {
        Self {
            setpoint: 0.0,
            output_high: 100.0,
            output_low: -100.0,
            hysteresis: 1.0,
            dead_band: 0.0,
            output: 0.0,
            last_state: BangBangState::Off,
        }
    }
}

impl BangBangController {
    pub fn new(setpoint: f64, output_high: f64, output_low: f64, hysteresis: f64) -> Self {
        Self {
            setpoint,
            output_high,
            output_low,
            hysteresis,
            ..Default::default()
        }
    }

    /// 计算 Bang-Bang 输出
    ///
    /// 带回滞的开关逻辑：
    /// - 误差 > hysteresis → 输出 output_high
    /// - 误差 < -hysteresis → 输出 output_low
    /// - 在回滞区内保持上一次输出
    pub fn compute(&mut self, feedback: f64) -> f64 {
        let error = self.setpoint - feedback;

        // 死区检查
        if error.abs() < self.dead_band {
            self.output = 0.0;
            self.last_state = BangBangState::Off;
            return self.output;
        }

        // 回滞逻辑
        if error > self.hysteresis {
            self.output = self.output_high;
            self.last_state = BangBangState::High;
        } else if error < -self.hysteresis {
            self.output = self.output_low;
            self.last_state = BangBangState::Low;
        }
        // else: 在回滞区内，保持 last_state 不变

        self.output
    }

    pub fn reset(&mut self) {
        self.output = 0.0;
        self.last_state = BangBangState::Off;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bangbang_default() {
        let c = BangBangController::default();
        assert_eq!(c.setpoint, 0.0);
        assert_eq!(c.output_high, 100.0);
        assert_eq!(c.output_low, -100.0);
        assert_eq!(c.hysteresis, 1.0);
        assert_eq!(c.output, 0.0);
    }

    #[test]
    fn test_bangbang_new() {
        let c = BangBangController::new(10.0, 50.0, -50.0, 2.0);
        assert_eq!(c.setpoint, 10.0);
        assert_eq!(c.output_high, 50.0);
        assert_eq!(c.output_low, -50.0);
        assert_eq!(c.hysteresis, 2.0);
    }

    #[test]
    fn test_positive_error_high_output() {
        let mut c = BangBangController::new(100.0, 50.0, -50.0, 1.0);
        let out = c.compute(0.0); // error=100 > hysteresis=1
        assert_eq!(out, 50.0);
        assert_eq!(c.last_state, BangBangState::High);
    }

    #[test]
    fn test_negative_error_low_output() {
        let mut c = BangBangController::new(0.0, 50.0, -50.0, 1.0);
        let out = c.compute(100.0); // error=-100 < -hysteresis=-1
        assert_eq!(out, -50.0);
        assert_eq!(c.last_state, BangBangState::Low);
    }

    #[test]
    fn test_hysteresis_hold() {
        let mut c = BangBangController::new(10.0, 100.0, -100.0, 5.0);
        // 大误差 → High
        c.compute(0.0); // error=10 > 5
        assert_eq!(c.last_state, BangBangState::High);

        // 误差缩小到回滞区内 → 应保持 High
        let out = c.compute(7.0); // error=3 < hysteresis=5
        assert_eq!(out, 100.0);
        assert_eq!(c.last_state, BangBangState::High);
    }

    #[test]
    fn test_hysteresis_switch() {
        let mut c = BangBangController::new(10.0, 100.0, -100.0, 2.0);
        c.compute(0.0); // error=10 → High
        assert_eq!(c.last_state, BangBangState::High);

        // 反向超过回滞
        let out = c.compute(20.0); // error=-10 < -2
        assert_eq!(out, -100.0);
        assert_eq!(c.last_state, BangBangState::Low);
    }

    #[test]
    fn test_dead_band() {
        let mut c = BangBangController::new(10.0, 100.0, -100.0, 1.0);
        c.dead_band = 5.0;
        let out = c.compute(8.0); // error=2 < dead_band=5
        assert_eq!(out, 0.0);
        assert_eq!(c.last_state, BangBangState::Off);
    }

    #[test]
    fn test_reset() {
        let mut c = BangBangController::new(10.0, 100.0, -100.0, 1.0);
        c.compute(0.0);
        assert_eq!(c.last_state, BangBangState::High);
        c.reset();
        assert_eq!(c.output, 0.0);
        assert_eq!(c.last_state, BangBangState::Off);
    }

    #[test]
    fn test_asymmetric_output() {
        let mut c = BangBangController::new(0.0, 80.0, -40.0, 0.5);
        c.compute(-10.0); // error=10 → High
        assert_eq!(c.output, 80.0);
        c.compute(10.0); // error=-10 → Low
        assert_eq!(c.output, -40.0);
    }
}
