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
    #[test]
    fn test_defaults_have_distinct_kp() {
        let presets = Preset::defaults();
        let kps: std::collections::HashSet<i64> = presets
            .iter()
            .map(|p| (p.kp * 100.0).round() as i64)
            .collect();
        assert_eq!(
            kps.len(),
            4,
            "default presets should have distinct kp values"
        );
    }

    #[test]
    fn test_defaults_ordered_by_aggressiveness() {
        let presets = Preset::defaults();
        for i in 0..presets.len() - 1 {
            assert!(
                presets[i].kp < presets[i + 1].kp,
                "presets should be ordered by increasing kp: {} < {}",
                presets[i].kp,
                presets[i + 1].kp
            );
        }
    }
    #[test]
    fn test_from_controller_captures_limits() {
        let pid = PidController::with_limits(1.0, 0.1, 0.01, 50.0, 200.0, 100.0);
        let preset = Preset::from_controller("L", "limits test", &pid);
        assert_eq!(preset.output_limit, 200.0);
        assert_eq!(preset.integral_limit, 100.0);
    }
    #[test]
    fn test_apply_to_overwrites() {
        let mut pid = PidController::new(99.0, 99.0, 99.0, 99.0);
        let preset = Preset::new("O", "overwrite", 1.0, 0.1, 0.01, 0.0, 100.0, 50.0);
        preset.apply_to(&mut pid);
        assert_eq!(pid.kp, 1.0, "kp should be overwritten");
        assert_eq!(pid.ki, 0.1, "ki should be overwritten");
        assert_eq!(pid.kd, 0.01, "kd should be overwritten");
    }
    #[test]
    fn test_serialization_all_fields() {
        let preset = Preset::new("Full", "all fields", 2.5, 0.25, 0.025, 15.0, 300.0, 150.0);
        let json = serde_json::to_string(&preset).unwrap();
        let d: Preset = serde_json::from_str(&json).unwrap();
        assert_eq!(d.name, "Full");
        assert_eq!(d.description, "all fields");
        assert_eq!(d.kp, 2.5);
        assert_eq!(d.ki, 0.25);
        assert_eq!(d.kd, 0.025);
        assert_eq!(d.setpoint, 15.0);
        assert_eq!(d.output_limit, 300.0);
        assert_eq!(d.integral_limit, 150.0);
    }
    #[test]
    fn test_new_accepts_string_and_str() {
        let p1 = Preset::new("name", "desc", 1.0, 0.1, 0.01, 0.0, 100.0, 50.0);
        let p2 = Preset::new(
            String::from("name"),
            String::from("desc"),
            1.0,
            0.1,
            0.01,
            0.0,
            100.0,
            50.0,
        );
        assert_eq!(p1.name, p2.name);
        assert_eq!(p1.description, p2.description);
    }
    #[test]
    fn test_defaults_have_positive_gains() {
        for preset in Preset::defaults() {
            assert!(preset.kp > 0.0, "{}: kp should be positive", preset.name);
            assert!(
                preset.ki >= 0.0,
                "{}: ki should be non-negative",
                preset.name
            );
            assert!(
                preset.kd >= 0.0,
                "{}: kd should be non-negative",
                preset.name
            );
            assert!(
                preset.output_limit > 0.0,
                "{}: output_limit should be positive",
                preset.name
            );
        }
    }
    #[test]
    fn preset_list_unique_names() {
        let list = Preset::defaults();
        let names: Vec<&str> = list.iter().map(|p| p.name.as_str()).collect();
        let unique: std::collections::HashSet<&str> = names.iter().copied().collect();
        assert_eq!(names.len(), unique.len());
    }

    #[test]
    fn preset_apply_sets_params() {
        let list = Preset::defaults();
        if let Some(preset) = list.first() {
            let mut pid = crate::models::PidController::default();
            preset.apply_to(&mut pid);
            assert!(pid.kp.is_finite());
            assert!(pid.ki.is_finite());
            assert!(pid.kd.is_finite());
        }
    }
    #[test]
    fn preset_defaults_count() {
        let list = Preset::defaults();
        assert!(!list.is_empty());
    }
}
