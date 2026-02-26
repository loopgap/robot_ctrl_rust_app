// ═══════════════════════════════════════════════════════════════
// 自抗扰控制器 (ADRC - Active Disturbance Rejection Control)
// ═══════════════════════════════════════════════════════════════
//
// 由韩京清提出，核心思想：
// 1. 跟踪微分器 (TD)：安排过渡过程，避免设定值突变
// 2. 扩张状态观测器 (ESO)：实时估计系统状态及总扰动
// 3. 非线性状态误差反馈 (NLSEF)：根据误差进行非线性组合
//
// 优点：不依赖精确数学模型，抗扰能力强
// 跨平台：仅使用 std::time::Instant

use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdrcController {
    // ── 设定值 ──
    pub setpoint: f64,
    pub output_limit: f64,

    // ── TD 参数 ──
    /// 跟踪微分器速度因子
    pub td_r: f64,
    /// 滤波因子
    pub td_h0: f64,

    // ── ESO 参数 ──
    /// 观测器带宽参数 β1, β2, β3
    pub eso_beta1: f64,
    pub eso_beta2: f64,
    pub eso_beta3: f64,
    /// 补偿增益 b0
    pub eso_b0: f64,

    // ── NLSEF 参数 ──
    /// 比例增益
    pub nlsef_beta1: f64,
    /// 微分增益
    pub nlsef_beta2: f64,
    /// 非线性阈值
    pub nlsef_delta: f64,
    /// 非线性指数 alpha1, alpha2
    pub nlsef_alpha1: f64,
    pub nlsef_alpha2: f64,

    // ── 内部状态 (serde skip) ──
    #[serde(skip)]
    pub td_v1: f64, // 跟踪信号
    #[serde(skip)]
    pub td_v2: f64, // 跟踪微分信号
    #[serde(skip)]
    pub eso_z1: f64, // 观测状态 1 (位置估计)
    #[serde(skip)]
    pub eso_z2: f64, // 观测状态 2 (速度估计)
    #[serde(skip)]
    pub eso_z3: f64, // 观测状态 3 (总扰动估计)
    #[serde(skip)]
    pub output: f64,
    #[serde(skip)]
    last_update: Option<Instant>,
}

impl Default for AdrcController {
    fn default() -> Self {
        Self {
            setpoint: 0.0,
            output_limit: 100.0,
            td_r: 100.0,
            td_h0: 0.01,
            eso_beta1: 100.0,
            eso_beta2: 300.0,
            eso_beta3: 1000.0,
            eso_b0: 1.0,
            nlsef_beta1: 10.0,
            nlsef_beta2: 5.0,
            nlsef_delta: 0.5,
            nlsef_alpha1: 0.75,
            nlsef_alpha2: 1.5,
            td_v1: 0.0,
            td_v2: 0.0,
            eso_z1: 0.0,
            eso_z2: 0.0,
            eso_z3: 0.0,
            output: 0.0,
            last_update: None,
        }
    }
}

impl AdrcController {
    pub fn new(setpoint: f64, b0: f64) -> Self {
        Self {
            setpoint,
            eso_b0: b0,
            ..Default::default()
        }
    }

    /// fhan 函数 - 最速离散跟踪微分器
    fn fhan(x1: f64, x2: f64, r: f64, h: f64) -> f64 {
        let d = r * h;
        let d0 = h * d;
        let y = x1 + h * x2;
        let a0 = (d * d + 8.0 * r * y.abs()).sqrt();
        let a = if y.abs() <= d0 {
            x2 + y / h
        } else {
            x2 + (a0 - d) / 2.0 * y.signum()
        };
        if a.abs() <= d {
            -r * a / d
        } else {
            -r * a.signum()
        }
    }

    /// fal 函数 - 非线性函数
    fn fal(e: f64, alpha: f64, delta: f64) -> f64 {
        if e.abs() <= delta {
            if delta > 1e-12 {
                e / delta.powf(1.0 - alpha)
            } else {
                e
            }
        } else {
            e.abs().powf(alpha) * e.signum()
        }
    }

    pub fn compute(&mut self, feedback: f64) -> f64 {
        let now = Instant::now();
        let dt = match self.last_update {
            Some(last) => now.duration_since(last).as_secs_f64(),
            None => {
                self.last_update = Some(now);
                self.eso_z1 = feedback;
                self.td_v1 = self.setpoint;
                return 0.0;
            }
        };
        if dt <= 0.0 {
            return self.output;
        }

        // ── 1. 跟踪微分器 (TD) ──
        let fh = Self::fhan(
            self.td_v1 - self.setpoint,
            self.td_v2,
            self.td_r,
            self.td_h0,
        );
        self.td_v1 += dt * self.td_v2;
        self.td_v2 += dt * fh;

        // ── 2. 扩张状态观测器 (ESO) ──
        let e_eso = self.eso_z1 - feedback;
        self.eso_z1 += dt * (self.eso_z2 - self.eso_beta1 * e_eso);
        self.eso_z2 += dt
            * (self.eso_z3 - self.eso_beta2 * Self::fal(e_eso, 0.5, self.nlsef_delta)
                + self.eso_b0 * self.output);
        self.eso_z3 += dt * (-self.eso_beta3 * Self::fal(e_eso, 0.25, self.nlsef_delta));

        // ── 3. NLSEF ──
        let e1 = self.td_v1 - self.eso_z1;
        let e2 = self.td_v2 - self.eso_z2;

        let u0 = self.nlsef_beta1 * Self::fal(e1, self.nlsef_alpha1, self.nlsef_delta)
            + self.nlsef_beta2 * Self::fal(e2, self.nlsef_alpha2, self.nlsef_delta);

        // 扰动补偿
        let u = if self.eso_b0.abs() > 1e-12 {
            (u0 - self.eso_z3) / self.eso_b0
        } else {
            u0
        };

        self.output = u.clamp(-self.output_limit, self.output_limit);
        self.last_update = Some(now);
        self.output
    }

    pub fn reset(&mut self) {
        self.td_v1 = 0.0;
        self.td_v2 = 0.0;
        self.eso_z1 = 0.0;
        self.eso_z2 = 0.0;
        self.eso_z3 = 0.0;
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
    fn test_adrc_default() {
        let c = AdrcController::default();
        assert_eq!(c.setpoint, 0.0);
        assert_eq!(c.output, 0.0);
        assert!(c.eso_b0 > 0.0);
    }

    #[test]
    fn test_adrc_new() {
        let c = AdrcController::new(10.0, 2.0);
        assert_eq!(c.setpoint, 10.0);
        assert_eq!(c.eso_b0, 2.0);
    }

    #[test]
    fn test_first_compute_returns_zero() {
        let mut c = AdrcController::new(50.0, 1.0);
        let out = c.compute(0.0);
        assert_eq!(out, 0.0);
    }

    #[test]
    fn test_adrc_positive_error() {
        let mut c = AdrcController::new(10.0, 1.0);
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
    fn test_adrc_output_limit() {
        let mut c = AdrcController::new(1000.0, 1.0);
        c.output_limit = 50.0;
        c.nlsef_beta1 = 100.0;
        c.compute(0.0);
        thread::sleep(Duration::from_millis(10));
        let out = c.compute(0.0);
        assert!(out <= 50.0, "Output should be clamped, got {}", out);
        assert!(out >= -50.0);
    }

    #[test]
    fn test_adrc_reset() {
        let mut c = AdrcController::new(10.0, 1.0);
        c.compute(0.0);
        thread::sleep(Duration::from_millis(10));
        c.compute(5.0);
        c.reset();
        assert_eq!(c.td_v1, 0.0);
        assert_eq!(c.eso_z1, 0.0);
        assert_eq!(c.eso_z3, 0.0);
        assert_eq!(c.output, 0.0);
        assert!(c.last_update.is_none());
    }

    #[test]
    fn test_fhan_function() {
        let f = AdrcController::fhan(1.0, 0.0, 100.0, 0.01);
        // 应返回有限值
        assert!(f.is_finite());
        assert!(f.abs() > 0.0);
    }

    #[test]
    fn test_fal_linear_region() {
        let v = AdrcController::fal(0.1, 0.75, 1.0);
        assert!(v.is_finite());
        assert!(v > 0.0);
    }

    #[test]
    fn test_fal_nonlinear_region() {
        let v = AdrcController::fal(5.0, 0.75, 1.0);
        assert!(v.is_finite());
        assert!(v > 0.0);
    }

    #[test]
    fn test_adrc_negative_error() {
        let mut c = AdrcController::new(-10.0, 1.0);
        c.compute(0.0);
        thread::sleep(Duration::from_millis(10));
        let out = c.compute(0.0);
        assert!(
            out < 0.0,
            "Should output negative for negative setpoint error, got {}",
            out
        );
    }
}
