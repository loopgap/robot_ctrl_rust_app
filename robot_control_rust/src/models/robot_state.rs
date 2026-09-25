use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobotState {
    pub timestamp: DateTime<Local>,
    pub position: f64,
    pub velocity: f64,
    pub current: f64,
    pub temperature: f64,
    pub pid_output: f64,
    pub error: f64,
    pub emergency_stop: bool,
    // 扩展字段
    pub acceleration: f64,
    pub voltage: f64,
    pub pwm_duty: f64,
    pub encoder_count: i64,
}

impl Default for RobotState {
    fn default() -> Self {
        Self {
            timestamp: Local::now(),
            position: 0.0,
            velocity: 0.0,
            current: 0.0,
            temperature: 0.0,
            pid_output: 0.0,
            error: 0.0,
            emergency_stop: false,
            acceleration: 0.0,
            voltage: 0.0,
            pwm_duty: 0.0,
            encoder_count: 0,
        }
    }
}

impl RobotState {
    pub fn new(position: f64, velocity: f64, current: f64, temperature: f64) -> Self {
        Self {
            timestamp: Local::now(),
            position,
            velocity,
            current,
            temperature,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_robot_state_default() {
        let s = RobotState::default();
        assert_eq!(s.position, 0.0);
        assert_eq!(s.velocity, 0.0);
        assert_eq!(s.current, 0.0);
        assert_eq!(s.temperature, 0.0);
        assert!(!s.emergency_stop);
        assert_eq!(s.pid_output, 0.0);
        assert_eq!(s.error, 0.0);
        assert_eq!(s.acceleration, 0.0);
        assert_eq!(s.voltage, 0.0);
        assert_eq!(s.pwm_duty, 0.0);
        assert_eq!(s.encoder_count, 0);
    }

    #[test]
    fn test_robot_state_new() {
        let s = RobotState::new(1.5, 2.0, 0.5, 25.0);
        assert_eq!(s.position, 1.5);
        assert_eq!(s.velocity, 2.0);
        assert_eq!(s.current, 0.5);
        assert_eq!(s.temperature, 25.0);
        assert_eq!(s.error, 0.0); // default
        assert_eq!(s.pid_output, 0.0); // default
        assert!(!s.emergency_stop); // default
    }

    #[test]
    fn test_robot_state_serialization_roundtrip() {
        let s = RobotState::new(10.0, 5.0, 1.0, 30.0);
        let json = serde_json::to_string(&s).unwrap();
        let s2: RobotState = serde_json::from_str(&json).unwrap();
        assert_eq!(s.position, s2.position);
        assert_eq!(s.velocity, s2.velocity);
        assert_eq!(s.current, s2.current);
        assert_eq!(s.temperature, s2.temperature);
    }

    #[test]
    fn test_robot_state_extreme_values() {
        let s = RobotState::new(f64::MAX, f64::MIN, f64::MAX, f64::MIN);
        assert_eq!(s.position, f64::MAX);
        assert_eq!(s.velocity, f64::MIN);
        let json = serde_json::to_string(&s).unwrap();
        let s2: RobotState = serde_json::from_str(&json).unwrap();
        assert_eq!(s.position, s2.position);
        assert_eq!(s.velocity, s2.velocity);
    }

    #[test]
    fn test_robot_state_negative_values() {
        let s = RobotState::new(-100.0, -50.0, -5.0, -40.0);
        assert_eq!(s.position, -100.0);
        assert_eq!(s.velocity, -50.0);
        assert_eq!(s.current, -5.0);
        assert_eq!(s.temperature, -40.0);
    }

    #[test]
    fn test_robot_state_mutation() {
        let s = RobotState {
            position: 42.0,
            velocity: std::f64::consts::PI,
            pid_output: 0.75,
            error: -0.5,
            emergency_stop: true,
            encoder_count: 12345,
            ..Default::default()
        };
        assert_eq!(s.position, 42.0);
        assert_eq!(s.velocity, std::f64::consts::PI);
        assert_eq!(s.pid_output, 0.75);
        assert_eq!(s.error, -0.5);
        assert!(s.emergency_stop);
        assert_eq!(s.encoder_count, 12345);
    }
    #[test]
    fn test_serialization_preserves_all_fields() {
        let s = RobotState {
            position: 1.0,
            velocity: 2.0,
            current: 3.0,
            temperature: 4.0,
            pid_output: 5.0,
            error: 6.0,
            emergency_stop: true,
            acceleration: 7.0,
            voltage: 48.0,
            pwm_duty: 75.5,
            encoder_count: 99999,
            ..Default::default()
        };
        let json = serde_json::to_string(&s).unwrap();
        let s2: RobotState = serde_json::from_str(&json).unwrap();
        assert_eq!(s2.position, 1.0);
        assert_eq!(s2.velocity, 2.0);
        assert_eq!(s2.current, 3.0);
        assert_eq!(s2.temperature, 4.0);
        assert_eq!(s2.pid_output, 5.0);
        assert_eq!(s2.error, 6.0);
        assert!(s2.emergency_stop);
        assert_eq!(s2.acceleration, 7.0);
        assert_eq!(s2.voltage, 48.0);
        assert_eq!(s2.pwm_duty, 75.5);
        assert_eq!(s2.encoder_count, 99999);
    }
    #[test]
    fn test_debug_format() {
        let s = RobotState::new(1.0, 2.0, 3.0, 4.0);
        let debug = format!("{:?}", s);
        assert!(
            debug.contains("position"),
            "Debug should contain field names"
        );
        assert!(
            debug.contains("velocity"),
            "Debug should contain field names"
        );
    }
    #[test]
    fn test_clone() {
        let s = RobotState::new(10.0, 20.0, 30.0, 40.0);
        let s2 = s.clone();
        assert_eq!(s.position, s2.position);
        assert_eq!(s.velocity, s2.velocity);
        assert_eq!(s.emergency_stop, s2.emergency_stop);
    }
    #[test]
    fn test_new_defaults_unused_fields() {
        let s = RobotState::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(s.pid_output, 0.0);
        assert_eq!(s.error, 0.0);
        assert!(!s.emergency_stop);
        assert_eq!(s.acceleration, 0.0);
        assert_eq!(s.voltage, 0.0);
        assert_eq!(s.pwm_duty, 0.0);
        assert_eq!(s.encoder_count, 0);
    }
}
