use super::pid_controller::PidController;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    pub name: String,
    pub description: String,
    pub kp: f64,
    pub ki: f64,
    pub kd: f64,
    pub setpoint: f64,
    pub output_limit: f64,
    pub integral_limit: f64,
}

impl Preset {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: impl Into<String>,
        desc: impl Into<String>,
        kp: f64,
        ki: f64,
        kd: f64,
        setpoint: f64,
        output_limit: f64,
        integral_limit: f64,
    ) -> Self {
        Self {
            name: name.into(),
            description: desc.into(),
            kp,
            ki,
            kd,
            setpoint,
            output_limit,
            integral_limit,
        }
    }

    pub fn from_controller(
        name: impl Into<String>,
        desc: impl Into<String>,
        c: &PidController,
    ) -> Self {
        Self {
            name: name.into(),
            description: desc.into(),
            kp: c.kp,
            ki: c.ki,
            kd: c.kd,
            setpoint: c.setpoint,
            output_limit: c.output_limit,
            integral_limit: c.integral_limit,
        }
    }

    pub fn apply_to(&self, c: &mut PidController) {
        c.kp = self.kp;
        c.ki = self.ki;
        c.kd = self.kd;
        c.setpoint = self.setpoint;
        c.output_limit = self.output_limit;
        c.integral_limit = self.integral_limit;
    }

    pub fn defaults() -> Vec<Preset> {
        vec![
            Preset::new(
                "Conservative",
                "精密定位 - 低增益稳定控制",
                0.5,
                0.05,
                0.01,
                0.0,
                50.0,
                50.0,
            ),
            Preset::new(
                "Balanced",
                "平衡响应速度和稳定性",
                1.0,
                0.1,
                0.05,
                0.0,
                100.0,
                100.0,
            ),
            Preset::new(
                "Fast Response",
                "快速运动控制 - 高增益高响应",
                2.0,
                0.2,
                0.1,
                0.0,
                200.0,
                200.0,
            ),
            Preset::new(
                "Ultra Aggressive",
                "极限响应 - 仅用于测试",
                5.0,
                0.5,
                0.3,
                0.0,
                500.0,
                500.0,
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults_non_empty() {
        let presets = Preset::defaults();
        assert_eq!(presets.len(), 4);
    }

    #[test]
    fn test_preset_apply_to() {
        let preset = Preset::new("Test", "desc", 3.0, 0.3, 0.03, 5.0, 200.0, 100.0);
        let mut pid = PidController::default();
        preset.apply_to(&mut pid);
        assert_eq!(pid.kp, 3.0);
        assert_eq!(pid.ki, 0.3);
        assert_eq!(pid.kd, 0.03);
        assert_eq!(pid.setpoint, 5.0);
        assert_eq!(pid.output_limit, 200.0);
        assert_eq!(pid.integral_limit, 100.0);
    }

    #[test]
    fn test_preset_from_controller() {
        let pid = PidController::new(2.0, 0.5, 0.1, 10.0);
        let preset = Preset::from_controller("Saved", "round-trip", &pid);
        assert_eq!(preset.kp, 2.0);
        assert_eq!(preset.ki, 0.5);
        assert_eq!(preset.kd, 0.1);
        assert_eq!(preset.setpoint, 10.0);
    }

    #[test]
    fn test_preset_roundtrip() {
        let original = PidController::with_limits(1.5, 0.3, 0.05, 8.0, 150.0, 75.0);
        let preset = Preset::from_controller("RT", "roundtrip", &original);
        let mut restored = PidController::default();
        preset.apply_to(&mut restored);
        assert_eq!(restored.kp, original.kp);
        assert_eq!(restored.ki, original.ki);
        assert_eq!(restored.kd, original.kd);
        assert_eq!(restored.setpoint, original.setpoint);
        assert_eq!(restored.output_limit, original.output_limit);
        assert_eq!(restored.integral_limit, original.integral_limit);
    }

    #[test]
    fn test_preset_serialization() {
        let preset = Preset::new("Ser", "d", 1.0, 0.1, 0.01, 0.0, 100.0, 50.0);
        let json = serde_json::to_string(&preset).unwrap();
        let deserialized: Preset = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "Ser");
        assert_eq!(deserialized.kp, 1.0);
    }
}
