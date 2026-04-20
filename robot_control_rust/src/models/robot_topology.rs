use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════
// 底盘类型
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChassisType {
    Differential, // 差速驱动 (两轮)
    Mecanum,      // 麦克纳姆轮 (全向四轮)
    Omni3,        // 三轮全向
    Omni4,        // 四轮全向
    Ackermann,    // 阿克曼转向 (类汽车)
    Tracked,      // 履带式
    Scara,        // SCARA 机械臂
    SixDofArm,    // 六轴机械臂
    DeltaRobot,   // 并联 Delta 机器人
    Custom,       // 自定义
}

impl ChassisType {
    pub fn all() -> &'static [ChassisType] {
        &[
            Self::Differential,
            Self::Mecanum,
            Self::Omni3,
            Self::Omni4,
            Self::Ackermann,
            Self::Tracked,
            Self::Scara,
            Self::SixDofArm,
            Self::DeltaRobot,
            Self::Custom,
        ]
    }

    pub fn description(&self) -> &str {
        match self {
            Self::Differential => "两轮差速底盘 - 简单可靠，适合室内导航",
            Self::Mecanum => "四麦克纳姆轮 - 全向移动，物流AGV首选",
            Self::Omni3 => "三全向轮底盘 - 紧凑灵活，120°分布",
            Self::Omni4 => "四全向轮底盘 - 稳定全向移动",
            Self::Ackermann => "阿克曼转向 - 类汽车结构，高速稳定",
            Self::Tracked => "履带式底盘 - 强通过性，适合野外",
            Self::Scara => "SCARA机械臂 - 水平多关节，装配专用",
            Self::SixDofArm => "六自由度机械臂 - 通用工业抓取",
            Self::DeltaRobot => "Delta并联机器人 - 高速拣选",
            Self::Custom => "自定义拓扑 - 灵活配置",
        }
    }

    pub fn motor_count(&self) -> usize {
        match self {
            Self::Differential | Self::Tracked => 2,
            Self::Omni3 => 3,
            Self::Mecanum | Self::Omni4 | Self::Ackermann => 4,
            Self::Scara => 4,
            Self::SixDofArm => 6,
            Self::DeltaRobot => 3,
            Self::Custom => 0,
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            Self::Differential => "🚗",
            Self::Mecanum => "🏎",
            Self::Omni3 => "🔺",
            Self::Omni4 => "🔷",
            Self::Ackermann => "🚙",
            Self::Tracked => "🪖",
            Self::Scara => "🦾",
            Self::SixDofArm => "🤖",
            Self::DeltaRobot => "🕸",
            Self::Custom => "🔧",
        }
    }
}

impl std::fmt::Display for ChassisType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Differential => write!(f, "Differential Drive"),
            Self::Mecanum => write!(f, "Mecanum Wheel"),
            Self::Omni3 => write!(f, "3-Wheel Omni"),
            Self::Omni4 => write!(f, "4-Wheel Omni"),
            Self::Ackermann => write!(f, "Ackermann Steering"),
            Self::Tracked => write!(f, "Tracked"),
            Self::Scara => write!(f, "SCARA Arm"),
            Self::SixDofArm => write!(f, "6-DOF Arm"),
            Self::DeltaRobot => write!(f, "Delta Robot"),
            Self::Custom => write!(f, "Custom"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// 执行器类型
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActuatorType {
    DcMotor,
    BldcMotor,
    StepperMotor,
    Servo,
    LinearActuator,
    Pneumatic,
    Hydraulic,
}

impl ActuatorType {
    pub fn all() -> &'static [ActuatorType] {
        &[
            Self::DcMotor,
            Self::BldcMotor,
            Self::StepperMotor,
            Self::Servo,
            Self::LinearActuator,
            Self::Pneumatic,
            Self::Hydraulic,
        ]
    }
}

impl std::fmt::Display for ActuatorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DcMotor => write!(f, "DC Motor"),
            Self::BldcMotor => write!(f, "BLDC Motor"),
            Self::StepperMotor => write!(f, "Stepper Motor"),
            Self::Servo => write!(f, "Servo"),
            Self::LinearActuator => write!(f, "Linear Actuator"),
            Self::Pneumatic => write!(f, "Pneumatic"),
            Self::Hydraulic => write!(f, "Hydraulic"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// 电机/关节配置
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotorConfig {
    pub id: u8,
    pub name: String,
    pub actuator_type: ActuatorType,
    pub max_rpm: f64,
    pub max_current: f64,
    pub gear_ratio: f64,
    pub encoder_ppr: u32,
    pub reversed: bool,
    pub enabled: bool,
    // 运行时
    pub current_speed: f64,
    pub current_position: f64,
    pub target_speed: f64,
}

impl Default for MotorConfig {
    fn default() -> Self {
        Self {
            id: 0,
            name: "Motor".into(),
            actuator_type: ActuatorType::DcMotor,
            max_rpm: 3000.0,
            max_current: 10.0,
            gear_ratio: 1.0,
            encoder_ppr: 1024,
            reversed: false,
            enabled: true,
            current_speed: 0.0,
            current_position: 0.0,
            target_speed: 0.0,
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// 拓扑配置
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyConfig {
    pub chassis_type: ChassisType,
    pub motors: Vec<MotorConfig>,
    pub wheel_radius: f64,    // mm
    pub wheel_base: f64,      // mm (轴距)
    pub track_width: f64,     // mm (轮距)
    pub max_linear_vel: f64,  // mm/s
    pub max_angular_vel: f64, // rad/s
    pub name: String,
}

impl Default for TopologyConfig {
    fn default() -> Self {
        let chassis = ChassisType::Differential;
        let motors = (0..chassis.motor_count())
            .map(|i| MotorConfig {
                id: i as u8,
                name: format!("Motor_{}", i + 1),
                ..Default::default()
            })
            .collect();
        Self {
            chassis_type: chassis,
            motors,
            wheel_radius: 50.0,
            wheel_base: 300.0,
            track_width: 250.0,
            max_linear_vel: 1000.0,
            max_angular_vel: std::f64::consts::PI,
            name: "Default Robot".into(),
        }
    }
}

impl TopologyConfig {
    pub fn set_chassis_type(&mut self, ct: ChassisType) {
        self.chassis_type = ct;
        let count = ct.motor_count();
        self.motors.resize_with(count, Default::default);
        for (i, m) in self.motors.iter_mut().enumerate() {
            m.id = i as u8;
            if m.name.starts_with("Motor") {
                m.name = format!("Motor_{}", i + 1);
            }
        }
    }

    pub fn builtin_configs() -> Vec<TopologyConfig> {
        vec![
            {
                let mut c = TopologyConfig {
                    name: "RoboMaster Infantry".into(),
                    ..Default::default()
                };
                c.set_chassis_type(ChassisType::Mecanum);
                c.wheel_radius = 76.0;
                c.wheel_base = 400.0;
                c.track_width = 350.0;
                c
            },
            {
                let mut c = TopologyConfig {
                    name: "AGV Differential".into(),
                    ..Default::default()
                };
                c.set_chassis_type(ChassisType::Differential);
                c.wheel_radius = 65.0;
                c
            },
            {
                let mut c = TopologyConfig {
                    name: "6-DOF Manipulator".into(),
                    ..Default::default()
                };
                c.set_chassis_type(ChassisType::SixDofArm);
                for (i, m) in c.motors.iter_mut().enumerate() {
                    m.actuator_type = ActuatorType::Servo;
                    m.name = format!("Joint_{}", i + 1);
                }
                c
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chassis_all_count() {
        assert_eq!(ChassisType::all().len(), 10);
    }

    #[test]
    fn test_chassis_motor_count() {
        assert_eq!(ChassisType::Differential.motor_count(), 2);
        assert_eq!(ChassisType::Mecanum.motor_count(), 4);
        assert_eq!(ChassisType::Omni3.motor_count(), 3);
        assert_eq!(ChassisType::SixDofArm.motor_count(), 6);
        assert_eq!(ChassisType::DeltaRobot.motor_count(), 3);
        assert_eq!(ChassisType::Custom.motor_count(), 0);
    }

    #[test]
    fn test_chassis_display() {
        assert_eq!(
            format!("{}", ChassisType::Differential),
            "Differential Drive"
        );
        assert_eq!(format!("{}", ChassisType::Mecanum), "Mecanum Wheel");
    }

    #[test]
    fn test_chassis_icons_non_empty() {
        for ct in ChassisType::all() {
            assert!(!ct.icon().is_empty(), "{:?} should have icon", ct);
        }
    }

    #[test]
    fn test_chassis_descriptions_non_empty() {
        for ct in ChassisType::all() {
            assert!(
                !ct.description().is_empty(),
                "{:?} should have description",
                ct
            );
        }
    }

    #[test]
    fn test_actuator_all() {
        assert_eq!(ActuatorType::all().len(), 7);
    }

    #[test]
    fn test_topology_default() {
        let cfg = TopologyConfig::default();
        assert_eq!(cfg.chassis_type, ChassisType::Differential);
        assert_eq!(cfg.motors.len(), 2);
        assert!(cfg.wheel_radius > 0.0);
    }

    #[test]
    fn test_topology_set_chassis_type() {
        let mut cfg = TopologyConfig::default();
        cfg.set_chassis_type(ChassisType::Mecanum);
        assert_eq!(cfg.chassis_type, ChassisType::Mecanum);
        assert_eq!(cfg.motors.len(), 4);
        for (i, m) in cfg.motors.iter().enumerate() {
            assert_eq!(m.id, i as u8);
        }
    }

    #[test]
    fn test_topology_set_chassis_custom() {
        let mut cfg = TopologyConfig::default();
        cfg.set_chassis_type(ChassisType::Custom);
        assert_eq!(cfg.motors.len(), 0);
    }

    #[test]
    fn test_builtin_configs() {
        let configs = TopologyConfig::builtin_configs();
        assert!(!configs.is_empty());
        for c in &configs {
            assert!(!c.name.is_empty());
            assert_eq!(c.motors.len(), c.chassis_type.motor_count());
        }
    }

    #[test]
    fn test_motor_config_default() {
        let m = MotorConfig::default();
        assert!(m.max_rpm > 0.0);
        assert!(m.enabled);
        assert!(!m.reversed);
    }

    #[test]
    fn test_topology_serialization() {
        let cfg = TopologyConfig::default();
        let json = serde_json::to_string(&cfg).unwrap();
        let cfg2: TopologyConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(cfg.chassis_type, cfg2.chassis_type);
        assert_eq!(cfg.motors.len(), cfg2.motors.len());
    }
}
