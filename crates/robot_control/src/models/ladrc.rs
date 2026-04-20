// ═══════════════════════════════════════════════════════════════
// 线性自抗扰控制器 (LADRC - Linear ADRC)
// ═══════════════════════════════════════════════════════════════
//
// ADRC 的线性化简化版本（高志强, 2003）：
// - 线性扩张状态观测器 (LESO)：用线性增益代替非线性 fal
// - 线性状态误差反馈 (LSEF)：PD+扰动补偿
// - 仅需两个调参参数：观测器带宽 ωo 和控制器带宽 ωc
//
// 优点：参数整定简单，工程实用性强
// 跨平台：仅使用 std::time::Instant

use serde::{Deserialize, Serialize};
use std::time::Instant;

/// 系统阶数
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LadrcOrder {
    /// 一阶系统 (2 阶 LESO)
    First,
    /// 二阶系统 (3 阶 LESO)
    #[default]
    Second,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LadrcController {
    pub setpoint: f64,
    pub output_limit: f64,

    /// 系统阶数
    pub order: LadrcOrder,

    /// 控制器带宽 ωc
    pub omega_c: f64,
    /// 观测器带宽 ωo (通常 ωo = 3~5 × ωc)
    pub omega_o: f64,
    /// 补偿增益 b0
    pub b0: f64,

    // ── 内部状态 ──
    #[serde(skip)]
    pub z1: f64, // 状态估计 1
    #[serde(skip)]
    pub z2: f64, // 状态估计 2
    #[serde(skip)]
    pub z3: f64, // 总扰动估计 (仅二阶)
    #[serde(skip)]
    pub output: f64,
    #[serde(skip)]
    last_update: Option<Instant>,
}

impl Default for LadrcController {
    fn default() -> Self {
        Self {
            setpoint: 0.0,
            output_limit: 100.0,
            order: LadrcOrder::Second,
            omega_c: 10.0,
            omega_o: 50.0,
            b0: 1.0,
            z1: 0.0,
            z2: 0.0,
            z3: 0.0,
            output: 0.0,
            last_update: None,
        }
    }
}

impl LadrcController {
    pub fn new(setpoint: f64, omega_c: f64, omega_o: f64, b0: f64) -> Self {
        Self {
            setpoint,
            omega_c,
            omega_o,
            b0,
            ..Default::default()
        }
    }

    pub fn compute(&mut self, feedback: f64) -> f64 {
        let now = Instant::now();
        let dt = match self.last_update {
            Some(last) => now.duration_since(last).as_secs_f64(),
            None => {
                self.last_update = Some(now);
                self.z1 = feedback;
                return 0.0;
            }
        };
        if dt <= 0.0 {
            return self.output;
        }

        match self.order {
            LadrcOrder::First => self.compute_first_order(feedback, dt),
            LadrcOrder::Second => self.compute_second_order(feedback, dt),
        }
    }

    /// 一阶 LADRC: 2 阶 LESO + P 控制 + 扰动补偿
    fn compute_first_order(&mut self, feedback: f64, dt: f64) -> f64 {
        let wo = self.omega_o;

        // LESO (2阶)
        // β1 = 2ωo, β2 = ωo²
        let beta1 = 2.0 * wo;
        let beta2 = wo * wo;

        let e = self.z1 - feedback;
        self.z1 += dt * (self.z2 - beta1 * e + self.b0 * self.output);
        self.z2 += dt * (-beta2 * e);

        // LSEF (P 控制 + 扰动补偿)
        // kp = ωc
        let u0 = self.omega_c * (self.setpoint - self.z1);
        let u = if self.b0.abs() > 1e-12 {
            (u0 - self.z2) / self.b0
        } else {
            u0
        };

        self.output = u.clamp(-self.output_limit, self.output_limit);
        self.last_update = Some(Instant::now());
        self.output
    }

    /// 二阶 LADRC: 3 阶 LESO + PD 控制 + 扰动补偿
    fn compute_second_order(&mut self, feedback: f64, dt: f64) -> f64 {
        let wo = self.omega_o;

        // LESO (3阶)
        // β1 = 3ωo, β2 = 3ωo², β3 = ωo³
        let beta1 = 3.0 * wo;
        let beta2 = 3.0 * wo * wo;
        let beta3 = wo * wo * wo;

        let e = self.z1 - feedback;
        self.z1 += dt * (self.z2 - beta1 * e);
        self.z2 += dt * (self.z3 - beta2 * e + self.b0 * self.output);
        self.z3 += dt * (-beta3 * e);

        // LSEF (PD 控制 + 扰动补偿)
        // kp = ωc², kd = 2ωc
        let wc = self.omega_c;
        let u0 = wc * wc * (self.setpoint - self.z1) - 2.0 * wc * self.z2;
        let u = if self.b0.abs() > 1e-12 {
            (u0 - self.z3) / self.b0
        } else {
            u0
        };

        self.output = u.clamp(-self.output_limit, self.output_limit);
        self.last_update = Some(Instant::now());
        self.output
    }

    pub fn reset(&mut self) {
        self.z1 = 0.0;
        self.z2 = 0.0;
        self.z3 = 0.0;
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
    fn test_ladrc_default() {
        let c = LadrcController::default();
        assert_eq!(c.setpoint, 0.0);
        assert_eq!(c.output, 0.0);
        assert_eq!(c.order, LadrcOrder::Second);
    }

    #[test]
    fn test_ladrc_new() {
        let c = LadrcController::new(10.0, 20.0, 100.0, 2.0);
        assert_eq!(c.setpoint, 10.0);
        assert_eq!(c.omega_c, 20.0);
        assert_eq!(c.omega_o, 100.0);
        assert_eq!(c.b0, 2.0);
    }

    #[test]
    fn test_first_compute_returns_zero() {
        let mut c = LadrcController::new(50.0, 10.0, 50.0, 1.0);
        let out = c.compute(0.0);
        assert_eq!(out, 0.0);
    }

    #[test]
    fn test_ladrc_second_order_positive() {
        let mut c = LadrcController::new(10.0, 10.0, 50.0, 1.0);
        c.order = LadrcOrder::Second;
        c.compute(0.0);
        thread::sleep(Duration::from_millis(10));
        let out = c.compute(0.0);
        assert!(out > 0.0, "Should output positive, got {}", out);
    }

    #[test]
    fn test_ladrc_first_order_positive() {
        let mut c = LadrcController::new(10.0, 10.0, 50.0, 1.0);
        c.order = LadrcOrder::First;
        c.compute(0.0);
        thread::sleep(Duration::from_millis(10));
        let out = c.compute(0.0);
        assert!(out > 0.0, "Should output positive, got {}", out);
    }

    #[test]
    fn test_ladrc_output_limit() {
        let mut c = LadrcController::new(1000.0, 50.0, 200.0, 1.0);
        c.output_limit = 30.0;
        c.compute(0.0);
        thread::sleep(Duration::from_millis(10));
        let out = c.compute(0.0);
        assert!(
            (-30.0..=30.0).contains(&out),
            "Output should be clamped, got {}",
            out
        );
    }

    #[test]
    fn test_ladrc_reset() {
        let mut c = LadrcController::new(10.0, 10.0, 50.0, 1.0);
        c.compute(0.0);
        thread::sleep(Duration::from_millis(10));
        c.compute(5.0);
        c.reset();
        assert_eq!(c.z1, 0.0);
        assert_eq!(c.z2, 0.0);
        assert_eq!(c.z3, 0.0);
        assert_eq!(c.output, 0.0);
        assert!(c.last_update.is_none());
    }

    #[test]
    fn test_ladrc_negative_error() {
        let mut c = LadrcController::new(-10.0, 10.0, 50.0, 1.0);
        c.compute(0.0);
        thread::sleep(Duration::from_millis(10));
        let out = c.compute(0.0);
        assert!(out < 0.0, "Should output negative, got {}", out);
    }

    #[test]
    fn test_ladrc_order_serialization() {
        let t = LadrcOrder::First;
        let json = serde_json::to_string(&t).unwrap();
        let restored: LadrcOrder = serde_json::from_str(&json).unwrap();
        assert_eq!(t, restored);
    }
}
