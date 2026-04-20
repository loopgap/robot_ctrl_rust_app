// ═══════════════════════════════════════════════════════════════
// 控制算法统一接口与枚举
// ═══════════════════════════════════════════════════════════════

use serde::{Deserialize, Serialize};

/// 控制算法类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlAlgorithmType {
    /// 经典位置式 PID
    ClassicPid,
    /// 增量式 PID
    IncrementalPid,
    /// Bang-Bang (开关控制)
    BangBang,
    /// 模糊 PID (自适应)
    FuzzyPid,
    /// 串级 PID (内外环)
    CascadePid,
    /// Smith 预估控制 (带时滞补偿)
    SmithPredictor,
    /// 自抗扰控制 (ADRC)
    Adrc,
    /// 线性自抗扰控制 (LADRC)
    Ladrc,
    /// 线性二次调节器 (LQR)
    Lqr,
    /// 模型预测控制 (MPC)
    Mpc,
}

impl ControlAlgorithmType {
    /// 所有可用的算法类型
    pub fn all() -> &'static [Self] {
        &[
            Self::ClassicPid,
            Self::IncrementalPid,
            Self::BangBang,
            Self::FuzzyPid,
            Self::CascadePid,
            Self::SmithPredictor,
            Self::Adrc,
            Self::Ladrc,
            Self::Lqr,
            Self::Mpc,
        ]
    }

    /// 英文名称
    pub fn name_en(&self) -> &'static str {
        match self {
            Self::ClassicPid => "Classic PID",
            Self::IncrementalPid => "Incremental PID",
            Self::BangBang => "Bang-Bang",
            Self::FuzzyPid => "Fuzzy PID",
            Self::CascadePid => "Cascade PID",
            Self::SmithPredictor => "Smith Predictor",
            Self::Adrc => "ADRC",
            Self::Ladrc => "LADRC",
            Self::Lqr => "LQR",
            Self::Mpc => "MPC",
        }
    }

    /// 中文名称
    pub fn name_zh(&self) -> &'static str {
        match self {
            Self::ClassicPid => "经典PID",
            Self::IncrementalPid => "增量式PID",
            Self::BangBang => "Bang-Bang开关控制",
            Self::FuzzyPid => "模糊PID",
            Self::CascadePid => "串级PID",
            Self::SmithPredictor => "Smith预估控制",
            Self::Adrc => "自抗扰控制(ADRC)",
            Self::Ladrc => "线性自抗扰控制(LADRC)",
            Self::Lqr => "线性二次调节器(LQR)",
            Self::Mpc => "模型预测控制(MPC)",
        }
    }

    /// 英文描述
    pub fn desc_en(&self) -> &'static str {
        match self {
            Self::ClassicPid => "Standard positional PID with derivative filter, anti-windup, feedforward, and dead zone",
            Self::IncrementalPid => "Outputs control increment (delta u) instead of absolute value; bumpless transfer",
            Self::BangBang => "On-off threshold control with configurable hysteresis; simple and robust",
            Self::FuzzyPid => "Adaptive PID using fuzzy logic rules to auto-tune Kp/Ki/Kd based on error dynamics",
            Self::CascadePid => "Dual-loop PID: outer loop (position) + inner loop (velocity) for tighter control",
            Self::SmithPredictor => "PID with dead-time compensation via internal process model prediction",
            Self::Adrc => "Active Disturbance Rejection Control: TD + ESO + NLSEF for robust disturbance handling",
            Self::Ladrc => "Linear ADRC with bandwidth parameterization; simpler tuning than nonlinear ADRC",
            Self::Lqr => "Linear Quadratic Regulator: optimal state feedback with Riccati equation solution",
            Self::Mpc => "Model Predictive Control: receding horizon optimization with constraints",
        }
    }

    /// 中文描述
    pub fn desc_zh(&self) -> &'static str {
        match self {
            Self::ClassicPid => "标准位置式PID，支持微分滤波、抗积分饱和、前馈和死区",
            Self::IncrementalPid => "输出控制增量(Δu)而非绝对值，支持无扰切换",
            Self::BangBang => "开关阈值控制，可配置回滞区间，结构简单鲁棒",
            Self::FuzzyPid => "基于模糊逻辑规则自适应整定Kp/Ki/Kd，适应误差动态变化",
            Self::CascadePid => "双环PID：外环(位置)+内环(速度)，提升控制精度",
            Self::SmithPredictor => "PID结合内部过程模型预测，补偿时滞/纯延迟",
            Self::Adrc => "自抗扰控制：TD跟踪微分器 + ESO扩展状态观测器 + NLSEF非线性控制律",
            Self::Ladrc => "线性自抗扰：带宽参数化设计，比非线性ADRC更易整定",
            Self::Lqr => "线性二次调节器：通过Riccati方程求解最优状态反馈增益",
            Self::Mpc => "模型预测控制：滚动时域优化，支持约束处理",
        }
    }
}

impl std::fmt::Display for ControlAlgorithmType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name_en())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_types_count() {
        assert_eq!(ControlAlgorithmType::all().len(), 10);
    }

    #[test]
    fn test_names_non_empty() {
        for t in ControlAlgorithmType::all() {
            assert!(!t.name_en().is_empty());
            assert!(!t.name_zh().is_empty());
            assert!(!t.desc_en().is_empty());
            assert!(!t.desc_zh().is_empty());
        }
    }

    #[test]
    fn test_display() {
        assert_eq!(
            format!("{}", ControlAlgorithmType::ClassicPid),
            "Classic PID"
        );
        assert_eq!(format!("{}", ControlAlgorithmType::FuzzyPid), "Fuzzy PID");
    }

    #[test]
    fn test_serialization() {
        let t = ControlAlgorithmType::IncrementalPid;
        let json = serde_json::to_string(&t).unwrap();
        let restored: ControlAlgorithmType = serde_json::from_str(&json).unwrap();
        assert_eq!(t, restored);
    }
}
