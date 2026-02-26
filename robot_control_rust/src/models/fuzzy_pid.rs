// ═══════════════════════════════════════════════════════════════
// 模糊 PID 控制器
// ═══════════════════════════════════════════════════════════════
//
// 基于误差(e)和误差变化率(ec)的模糊逻辑规则自适应整定 PID 参数。
// 模糊集合: NB, NM, NS, ZO, PS, PM, PB (7级量化)
// 通过查表获取 ΔKp, ΔKi, ΔKd 的调整量，叠加到基础 PID 参数上。
// 纯 Rust 实现，无外部依赖，跨平台。

use serde::{Deserialize, Serialize};
use std::time::Instant;

/// 模糊语言变量 7 级
const FUZZY_LEVELS: usize = 7;

/// 模糊 PID 控制器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuzzyPidController {
    // 基础 PID 参数
    pub kp_base: f64,
    pub ki_base: f64,
    pub kd_base: f64,
    pub setpoint: f64,
    pub output_limit: f64,
    pub integral_limit: f64,

    // 模糊调整范围
    pub kp_range: f64, // Kp 最大调整量
    pub ki_range: f64, // Ki 最大调整量
    pub kd_range: f64, // Kd 最大调整量

    // 误差量化参数
    pub error_scale: f64, // 误差量化比例（用于映射到 [-3, 3]）
    pub ec_scale: f64,    // 误差变化率量化比例

    // 当前有效参数
    #[serde(skip)]
    pub effective_kp: f64,
    #[serde(skip)]
    pub effective_ki: f64,
    #[serde(skip)]
    pub effective_kd: f64,

    // PID 运行状态
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

    pub anti_windup: bool,
    pub derivative_filter: f64,
}

impl Default for FuzzyPidController {
    fn default() -> Self {
        Self {
            kp_base: 1.0,
            ki_base: 0.1,
            kd_base: 0.01,
            setpoint: 0.0,
            output_limit: 100.0,
            integral_limit: 100.0,
            kp_range: 1.0,
            ki_range: 0.1,
            kd_range: 0.05,
            error_scale: 10.0,
            ec_scale: 10.0,
            effective_kp: 1.0,
            effective_ki: 0.1,
            effective_kd: 0.01,
            integral: 0.0,
            last_error: 0.0,
            derivative: 0.0,
            last_update: None,
            output: 0.0,
            anti_windup: true,
            derivative_filter: 0.1,
        }
    }
}

impl FuzzyPidController {
    pub fn new(kp: f64, ki: f64, kd: f64, setpoint: f64) -> Self {
        Self {
            kp_base: kp,
            ki_base: ki,
            kd_base: kd,
            setpoint,
            effective_kp: kp,
            effective_ki: ki,
            effective_kd: kd,
            ..Default::default()
        }
    }

    /// 模糊规则表: ΔKp
    /// 行=误差e(NB..PB), 列=误差变化率ec(NB..PB)
    /// 值范围 [-3, 3] 映射到 [-kp_range, kp_range]
    const RULE_KP: [[i8; FUZZY_LEVELS]; FUZZY_LEVELS] = [
        // ec: NB  NM  NS  ZO  PS  PM  PB
        [3, 3, 2, 2, 1, 0, 0],       // e=NB
        [3, 3, 2, 1, 1, 0, -1],      // e=NM
        [2, 2, 1, 1, 0, -1, -1],     // e=NS
        [2, 1, 1, 0, -1, -1, -2],    // e=ZO
        [1, 1, 0, -1, -1, -2, -2],   // e=PS
        [0, 0, -1, -1, -2, -3, -3],  // e=PM
        [0, -1, -1, -2, -2, -3, -3], // e=PB
    ];

    /// 模糊规则表: ΔKi
    const RULE_KI: [[i8; FUZZY_LEVELS]; FUZZY_LEVELS] = [
        // ec: NB  NM  NS  ZO  PS  PM  PB
        [-3, -3, -2, -2, -1, 0, 0], // e=NB
        [-3, -3, -2, -1, -1, 0, 0], // e=NM
        [-2, -2, -1, 0, 0, 1, 1],   // e=NS
        [-2, -1, 0, 1, 1, 1, 2],    // e=ZO
        [-1, 0, 0, 1, 1, 2, 2],     // e=PS
        [0, 0, 1, 1, 2, 3, 3],      // e=PM
        [0, 0, 1, 2, 2, 3, 3],      // e=PB
    ];

    /// 模糊规则表: ΔKd
    const RULE_KD: [[i8; FUZZY_LEVELS]; FUZZY_LEVELS] = [
        // ec: NB  NM  NS  ZO  PS  PM  PB
        [3, 2, 1, 1, 0, -1, -3],  // e=NB
        [3, 2, 1, 1, 0, -1, -3],  // e=NM
        [2, 1, 0, 0, -1, -1, -2], // e=NS
        [1, 1, 0, 0, 0, -1, -1],  // e=ZO
        [-2, -1, -1, 0, 0, 1, 2], // e=PS
        [-3, -1, 0, 1, 1, 2, 3],  // e=PM
        [-3, -1, 0, 1, 1, 2, 3],  // e=PB
    ];

    /// 将连续值量化为模糊域索引 [0, 6]
    fn quantize(value: f64, scale: f64) -> (usize, usize, f64) {
        let normalized = (value / scale).clamp(-3.0, 3.0);
        let shifted = normalized + 3.0; // [0, 6]
        let low = (shifted.floor() as usize).min(5);
        let frac = shifted - shifted.floor();
        let high = if frac < 1e-9 { low } else { (low + 1).min(6) };
        (low, high, frac)
    }

    /// 从规则表中进行模糊推理（线性插值）
    fn fuzzy_lookup(
        table: &[[i8; FUZZY_LEVELS]; FUZZY_LEVELS],
        e_low: usize,
        e_high: usize,
        e_frac: f64,
        ec_low: usize,
        ec_high: usize,
        ec_frac: f64,
    ) -> f64 {
        let v00 = table[e_low][ec_low] as f64;
        let v01 = table[e_low][ec_high] as f64;
        let v10 = table[e_high][ec_low] as f64;
        let v11 = table[e_high][ec_high] as f64;

        let v0 = v00 + (v01 - v00) * ec_frac;
        let v1 = v10 + (v11 - v10) * ec_frac;
        v0 + (v1 - v0) * e_frac
    }

    /// 模糊推理更新 PID 参数
    fn fuzzy_tune(&mut self, error: f64, error_change: f64) {
        let (e_low, e_high, e_frac) = Self::quantize(error, self.error_scale);
        let (ec_low, ec_high, ec_frac) = Self::quantize(error_change, self.ec_scale);

        let delta_kp = Self::fuzzy_lookup(
            &Self::RULE_KP,
            e_low,
            e_high,
            e_frac,
            ec_low,
            ec_high,
            ec_frac,
        );
        let delta_ki = Self::fuzzy_lookup(
            &Self::RULE_KI,
            e_low,
            e_high,
            e_frac,
            ec_low,
            ec_high,
            ec_frac,
        );
        let delta_kd = Self::fuzzy_lookup(
            &Self::RULE_KD,
            e_low,
            e_high,
            e_frac,
            ec_low,
            ec_high,
            ec_frac,
        );

        // 将 [-3,3] 映射到 [-range, range]
        self.effective_kp = (self.kp_base + delta_kp / 3.0 * self.kp_range).max(0.0);
        self.effective_ki = (self.ki_base + delta_ki / 3.0 * self.ki_range).max(0.0);
        self.effective_kd = (self.kd_base + delta_kd / 3.0 * self.kd_range).max(0.0);
    }

    pub fn compute(&mut self, feedback: f64) -> f64 {
        let now = Instant::now();
        let dt = match self.last_update {
            Some(last) => now.duration_since(last).as_secs_f64(),
            None => {
                self.last_update = Some(now);
                self.last_error = self.setpoint - feedback;
                return 0.0;
            }
        };
        if dt <= 0.0 {
            return self.output;
        }

        let error = self.setpoint - feedback;
        let error_change = if dt > 0.0 {
            (error - self.last_error) / dt
        } else {
            0.0
        };

        // 模糊推理调整参数
        self.fuzzy_tune(error, error_change);

        // P
        let p_term = self.effective_kp * error;

        // I
        self.integral += error * dt;
        if self.anti_windup {
            self.integral = self
                .integral
                .clamp(-self.integral_limit, self.integral_limit);
        }
        let i_term = self.effective_ki * self.integral;

        // D (带滤波)
        let raw_deriv = if dt > 0.0 {
            (error - self.last_error) / dt
        } else {
            0.0
        };
        self.derivative =
            self.derivative * (1.0 - self.derivative_filter) + raw_deriv * self.derivative_filter;
        let d_term = self.effective_kd * self.derivative;

        let mut output = p_term + i_term + d_term;
        output = output.clamp(-self.output_limit, self.output_limit);

        // 抗积分饱和
        if self.anti_windup && output.abs() >= self.output_limit {
            self.integral -= error * dt;
        }

        self.last_error = error;
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
        self.effective_kp = self.kp_base;
        self.effective_ki = self.ki_base;
        self.effective_kd = self.kd_base;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_fuzzy_pid_default() {
        let c = FuzzyPidController::default();
        assert_eq!(c.kp_base, 1.0);
        assert_eq!(c.ki_base, 0.1);
        assert_eq!(c.kd_base, 0.01);
        assert_eq!(c.output, 0.0);
        assert!(c.anti_windup);
    }

    #[test]
    fn test_fuzzy_pid_new() {
        let c = FuzzyPidController::new(2.0, 0.5, 0.1, 10.0);
        assert_eq!(c.kp_base, 2.0);
        assert_eq!(c.setpoint, 10.0);
        assert_eq!(c.effective_kp, 2.0);
    }

    #[test]
    fn test_first_compute_returns_zero() {
        let mut c = FuzzyPidController::new(1.0, 0.0, 0.0, 100.0);
        let out = c.compute(0.0);
        assert_eq!(out, 0.0);
    }

    #[test]
    fn test_fuzzy_positive_error() {
        let mut c = FuzzyPidController::new(2.0, 0.0, 0.0, 10.0);
        c.error_scale = 10.0;
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
    fn test_fuzzy_params_adapt() {
        let mut c = FuzzyPidController::new(1.0, 0.1, 0.01, 20.0);
        c.kp_range = 2.0;
        c.ki_range = 0.5;
        c.kd_range = 0.1;
        c.error_scale = 10.0;
        c.ec_scale = 10.0;

        c.compute(0.0);
        thread::sleep(Duration::from_millis(10));
        c.compute(0.0);

        // 模糊推理应该自适应调整参数（effective != base）
        assert!(
            (c.effective_kp - c.kp_base).abs() > 0.01,
            "Kp should be adapted by fuzzy rules: effective={}, base={}",
            c.effective_kp,
            c.kp_base
        );
    }

    #[test]
    fn test_fuzzy_output_limit() {
        let mut c = FuzzyPidController::new(100.0, 0.0, 0.0, 100.0);
        c.output_limit = 50.0;
        c.compute(0.0);
        thread::sleep(Duration::from_millis(10));
        let out = c.compute(0.0);
        assert!(out <= 50.0, "Output should be clamped to 50.0, got {}", out);
    }

    #[test]
    fn test_fuzzy_reset() {
        let mut c = FuzzyPidController::new(1.0, 0.5, 0.1, 10.0);
        c.kp_range = 2.0;
        c.compute(0.0);
        thread::sleep(Duration::from_millis(10));
        c.compute(5.0);
        c.reset();
        assert_eq!(c.integral, 0.0);
        assert_eq!(c.last_error, 0.0);
        assert_eq!(c.output, 0.0);
        assert_eq!(c.effective_kp, c.kp_base);
        assert_eq!(c.effective_ki, c.ki_base);
        assert_eq!(c.effective_kd, c.kd_base);
    }

    #[test]
    fn test_quantize_boundaries() {
        let (low, high, frac) = FuzzyPidController::quantize(0.0, 10.0);
        assert_eq!(low, 3);
        assert_eq!(high, 3);
        assert!((frac - 0.0).abs() < 0.001);

        // 最大正 (clamped at 3.0, shifted=6.0, lands on index 5)
        let (low, high, _frac) = FuzzyPidController::quantize(100.0, 10.0);
        assert_eq!(low, 5);
        assert_eq!(high, 5);

        // 最大负
        let (low, _high, _frac) = FuzzyPidController::quantize(-100.0, 10.0);
        assert_eq!(low, 0);
    }

    #[test]
    fn test_fuzzy_lookup_center() {
        // e=ZO, ec=ZO → table[3][3] = 0 for ΔKp
        let val =
            FuzzyPidController::fuzzy_lookup(&FuzzyPidController::RULE_KP, 3, 3, 0.0, 3, 3, 0.0);
        assert!((val - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_fuzzy_lookup_corner() {
        // e=NB, ec=NB → table[0][0] = 3 for ΔKp
        let val =
            FuzzyPidController::fuzzy_lookup(&FuzzyPidController::RULE_KP, 0, 0, 0.0, 0, 0, 0.0);
        assert!((val - 3.0).abs() < 0.001);
    }

    #[test]
    fn test_fuzzy_negative_error() {
        let mut c = FuzzyPidController::new(1.0, 0.0, 0.0, -10.0);
        c.compute(0.0);
        thread::sleep(Duration::from_millis(10));
        let out = c.compute(0.0);
        assert!(
            out < 0.0,
            "Should output negative for negative error, got {}",
            out
        );
    }
}
