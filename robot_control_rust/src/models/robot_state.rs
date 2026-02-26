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
    }

    #[test]
    fn test_robot_state_new() {
        let s = RobotState::new(1.5, 2.0, 0.5, 25.0);
        assert_eq!(s.position, 1.5);
        assert_eq!(s.velocity, 2.0);
        assert_eq!(s.current, 0.5);
        assert_eq!(s.temperature, 25.0);
        assert_eq!(s.error, 0.0); // default
    }

    #[test]
    fn test_robot_state_serialization() {
        let s = RobotState::new(10.0, 5.0, 1.0, 30.0);
        let json = serde_json::to_string(&s).unwrap();
        let s2: RobotState = serde_json::from_str(&json).unwrap();
        assert_eq!(s.position, s2.position);
        assert_eq!(s.velocity, s2.velocity);
    }
}
