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
        // Verify serialization survives extreme values
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
        let mut s = RobotState::default();
        s.position = 42.0;
        s.velocity = 3.14;
        s.pid_output = 0.75;
        s.error = -0.5;
        s.emergency_stop = true;
        s.encoder_count = 12345;
        assert_eq!(s.position, 42.0);
        assert_eq!(s.velocity, 3.14);
        assert_eq!(s.pid_output, 0.75);
        assert_eq!(s.error, -0.5);
        assert!(s.emergency_stop);
        assert_eq!(s.encoder_count, 12345);
    }
}
