// ═══════════════════════════════════════════════════════════════
// 底盘运动学 (Chassis Kinematics)
// ═══════════════════════════════════════════════════════════════
//
// 为各种机器人底盘提供正/逆运动学解算与控制代码示例
// - 差速驱动 (Differential Drive)
// - 麦克纳姆轮 (Mecanum)
// - 三轮全向 (Omni-3)
// - 四轮全向 (Omni-4)
// - 阿克曼 (Ackermann)
// - 履带式 (Tracked, 等同差速)
//
// 所有计算纯 Rust，跨平台，无外部依赖

use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════
// 通用速度结构
// ═══════════════════════════════════════════════════════════════

/// 底盘速度指令 (vx, vy, omega)
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct ChassisVelocity {
    pub vx: f64,    // 前进速度 mm/s
    pub vy: f64,    // 横向速度 mm/s (差速/阿克曼/履带为 0)
    pub omega: f64, // 旋转角速度 rad/s
}

/// 各轮转速 (最多支持 6 轮)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WheelSpeeds {
    pub speeds: Vec<f64>, // 各轮角速度 rad/s
}

// ═══════════════════════════════════════════════════════════════
// 底盘运动学计算器
// ═══════════════════════════════════════════════════════════════

/// 底盘运动学计算
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChassisKinematics;

impl ChassisKinematics {
    // ─── 差速驱动 (Differential Drive) ───────────────────

    /// 逆运动学: (vx, omega) → (left_ω, right_ω)
    pub fn differential_inverse(
        vx: f64,
        omega: f64,
        wheel_radius: f64,
        track_width: f64,
    ) -> WheelSpeeds {
        let r = wheel_radius.max(1.0);
        let d = track_width / 2.0;
        WheelSpeeds {
            speeds: vec![
                (vx - omega * d) / r, // 左轮
                (vx + omega * d) / r, // 右轮
            ],
        }
    }

    /// 正运动学: (left_ω, right_ω) → (vx, omega)
    pub fn differential_forward(
        left_w: f64,
        right_w: f64,
        wheel_radius: f64,
        track_width: f64,
    ) -> ChassisVelocity {
        let r = wheel_radius;
        let d = track_width / 2.0;
        ChassisVelocity {
            vx: r * (left_w + right_w) / 2.0,
            vy: 0.0,
            omega: if d > 0.0 {
                r * (right_w - left_w) / (2.0 * d)
            } else {
                0.0
            },
        }
    }

    // ─── 麦克纳姆轮 (Mecanum 4-Wheel) ───────────────────

    /// 逆运动学: (vx, vy, omega) → 4 轮角速度
    /// 轮序: 左前, 右前, 左后, 右后
    /// 标准 X 排列麦克纳姆
    pub fn mecanum_inverse(
        vx: f64,
        vy: f64,
        omega: f64,
        wheel_radius: f64,
        wheel_base: f64,
        track_width: f64,
    ) -> WheelSpeeds {
        let r = wheel_radius.max(1.0);
        let k = (wheel_base + track_width) / 2.0;
        WheelSpeeds {
            speeds: vec![
                (vx - vy - omega * k) / r, // 左前 (fl)
                (vx + vy + omega * k) / r, // 右前 (fr)
                (vx + vy - omega * k) / r, // 左后 (rl)
                (vx - vy + omega * k) / r, // 右后 (rr)
            ],
        }
    }

    /// 正运动学: 4 轮 → (vx, vy, omega)
    pub fn mecanum_forward(
        fl: f64,
        fr: f64,
        rl: f64,
        rr: f64,
        wheel_radius: f64,
        wheel_base: f64,
        track_width: f64,
    ) -> ChassisVelocity {
        let r = wheel_radius;
        let k = (wheel_base + track_width) / 2.0;
        ChassisVelocity {
            vx: r * (fl + fr + rl + rr) / 4.0,
            vy: r * (-fl + fr + rl - rr) / 4.0,
            omega: if k > 0.0 {
                r * (-fl + fr - rl + rr) / (4.0 * k)
            } else {
                0.0
            },
        }
    }

    // ─── 三轮全向 (Omni-3, 120° 分布) ──────────────────

    /// 逆运动学: (vx, vy, omega) → 3 轮角速度
    /// 轮 1: 前方 (0°), 轮 2: 左后 (120°), 轮 3: 右后 (240°)
    pub fn omni3_inverse(
        vx: f64,
        vy: f64,
        omega: f64,
        wheel_radius: f64,
        chassis_radius: f64,
    ) -> WheelSpeeds {
        let r = wheel_radius.max(1.0);
        let l = chassis_radius;
        let sin60 = (std::f64::consts::PI / 3.0).sin(); // √3/2
        let cos60 = (std::f64::consts::PI / 3.0).cos(); // 0.5

        WheelSpeeds {
            speeds: vec![
                (-vy + omega * l) / r,                      // 轮 1 (前)
                (vx * sin60 + vy * cos60 + omega * l) / r,  // 轮 2 (左后)
                (-vx * sin60 + vy * cos60 + omega * l) / r, // 轮 3 (右后)
            ],
        }
    }

    /// 正运动学: 3 轮 → (vx, vy, omega)
    pub fn omni3_forward(
        w1: f64,
        w2: f64,
        w3: f64,
        wheel_radius: f64,
        chassis_radius: f64,
    ) -> ChassisVelocity {
        let r = wheel_radius;
        let l = chassis_radius;
        let sin60 = (std::f64::consts::PI / 3.0).sin();

        ChassisVelocity {
            vx: r * (w2 - w3) / (2.0 * sin60),
            vy: r * (-2.0 * w1 + w2 + w3) / 3.0,
            omega: if l > 0.0 {
                r * (w1 + w2 + w3) / (3.0 * l)
            } else {
                0.0
            },
        }
    }

    // ─── 四轮全向 (Omni-4, 90° 分布) ───────────────────

    /// 逆运动学: (vx, vy, omega) → 4 轮角速度
    /// 轮序: 前左(45°), 前右(315°), 后左(135°), 后右(225°)
    pub fn omni4_inverse(
        vx: f64,
        vy: f64,
        omega: f64,
        wheel_radius: f64,
        chassis_radius: f64,
    ) -> WheelSpeeds {
        let r = wheel_radius.max(1.0);
        let l = chassis_radius;
        let s2 = std::f64::consts::FRAC_1_SQRT_2; // 1/√2

        WheelSpeeds {
            speeds: vec![
                (-vx * s2 + vy * s2 + omega * l) / r, // 前左
                (vx * s2 + vy * s2 + omega * l) / r,  // 前右
                (-vx * s2 - vy * s2 + omega * l) / r, // 后左
                (vx * s2 - vy * s2 + omega * l) / r,  // 后右
            ],
        }
    }

    // ─── 阿克曼转向 (Ackermann) ─────────────────────────

    /// 逆运动学: (vx, steering_angle) → (left_w, right_w, steering_left, steering_right)
    /// 返回: [left_rear_ω, right_rear_ω, left_steer_angle, right_steer_angle]
    pub fn ackermann_inverse(
        vx: f64,
        steering_angle: f64,
        wheel_radius: f64,
        wheel_base: f64,
        track_width: f64,
    ) -> (WheelSpeeds, f64, f64) {
        let r = wheel_radius.max(1.0);
        let l = wheel_base;
        let d = track_width;

        if steering_angle.abs() < 1e-6 {
            // 直行
            let w = vx / r;
            return (
                WheelSpeeds {
                    speeds: vec![w, w, 0.0, 0.0],
                },
                0.0,
                0.0,
            );
        }

        let turn_radius = l / steering_angle.tan();
        let _omega = vx / turn_radius;

        let left_rear_w = vx * (1.0 - d / (2.0 * turn_radius)) / r;
        let right_rear_w = vx * (1.0 + d / (2.0 * turn_radius)) / r;

        // 阿克曼几何：内外轮转角不同
        let steer_left = (l / (turn_radius - d / 2.0)).atan();
        let steer_right = (l / (turn_radius + d / 2.0)).atan();

        (
            WheelSpeeds {
                speeds: vec![left_rear_w, right_rear_w],
            },
            steer_left,
            steer_right,
        )
    }

    /// 正运动学: 后轮速度 + 转向角 → (vx, omega)
    pub fn ackermann_forward(
        left_rear_w: f64,
        right_rear_w: f64,
        steering_angle: f64,
        wheel_radius: f64,
        wheel_base: f64,
    ) -> ChassisVelocity {
        let r = wheel_radius;
        let vx = r * (left_rear_w + right_rear_w) / 2.0;
        let omega = if wheel_base > 0.0 {
            vx * steering_angle.tan() / wheel_base
        } else {
            0.0
        };

        ChassisVelocity { vx, vy: 0.0, omega }
    }

    // ─── 履带式 (Tracked - 等同差速) ────────────────────

    /// 逆运动学: 同差速驱动
    pub fn tracked_inverse(vx: f64, omega: f64, track_width: f64) -> WheelSpeeds {
        // 履带无轮半径概念，返回线速度
        let d = track_width / 2.0;
        WheelSpeeds {
            speeds: vec![
                vx - omega * d, // 左侧线速度
                vx + omega * d, // 右侧线速度
            ],
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// 底盘控制代码示例
// ═══════════════════════════════════════════════════════════════

/// 各底盘类型的代码模板
pub struct ChassisCodeExamples;

impl ChassisCodeExamples {
    pub fn get_example(chassis_type: &str, lang: &str) -> &'static str {
        match (chassis_type, lang) {
            ("Differential", "en") => DIFF_EXAMPLE_EN,
            ("Differential", "zh") => DIFF_EXAMPLE_ZH,
            ("Mecanum", "en") => MECANUM_EXAMPLE_EN,
            ("Mecanum", "zh") => MECANUM_EXAMPLE_ZH,
            ("Omni3", "en") => OMNI3_EXAMPLE_EN,
            ("Omni3", "zh") => OMNI3_EXAMPLE_ZH,
            ("Omni4", "en") => OMNI4_EXAMPLE_EN,
            ("Omni4", "zh") => OMNI4_EXAMPLE_ZH,
            ("Ackermann", "en") => ACKERMANN_EXAMPLE_EN,
            ("Ackermann", "zh") => ACKERMANN_EXAMPLE_ZH,
            ("Tracked", "en") => TRACKED_EXAMPLE_EN,
            ("Tracked", "zh") => TRACKED_EXAMPLE_ZH,
            _ => GENERIC_EXAMPLE,
        }
    }

    pub fn all_chassis_keys() -> &'static [&'static str] {
        &[
            "Differential",
            "Mecanum",
            "Omni3",
            "Omni4",
            "Ackermann",
            "Tracked",
        ]
    }
}

const DIFF_EXAMPLE_EN: &str = r#"// Differential Drive Control Example
// Two-wheel robot with independent left/right motors
//
// Forward Kinematics:
//   vx    = R * (wL + wR) / 2
//   omega = R * (wR - wL) / (2 * D)
//
// Inverse Kinematics:
//   wL = (vx - omega * D) / R
//   wR = (vx + omega * D) / R
//
// where R=wheel_radius, D=track_width/2

fn control_loop(vx: f64, omega: f64) {
    let r = 0.05;  // wheel radius (m)
    let d = 0.15;  // half track width (m)

    let w_left  = (vx - omega * d) / r;
    let w_right = (vx + omega * d) / r;

    set_motor_speed(LEFT_MOTOR,  w_left);
    set_motor_speed(RIGHT_MOTOR, w_right);
}

// PID velocity control per wheel
fn wheel_pid(target: f64, actual: f64, state: &mut PidState) -> f64 {
    let error = target - actual;
    state.integral += error * DT;
    let derivative = (error - state.last_error) / DT;
    state.last_error = error;
    KP * error + KI * state.integral + KD * derivative
}"#;

const DIFF_EXAMPLE_ZH: &str = r#"// 差速驱动底盘控制示例
// 双轮独立驱动机器人
//
// 正运动学:
//   vx    = R * (wL + wR) / 2
//   omega = R * (wR - wL) / (2 * D)
//
// 逆运动学:
//   wL = (vx - omega * D) / R
//   wR = (vx + omega * D) / R
//
// R=轮半径, D=轮距/2

fn control_loop(vx: f64, omega: f64) {
    let r = 0.05;  // 轮半径 (m)
    let d = 0.15;  // 半轮距 (m)

    let w_left  = (vx - omega * d) / r;
    let w_right = (vx + omega * d) / r;

    set_motor_speed(LEFT_MOTOR,  w_left);
    set_motor_speed(RIGHT_MOTOR, w_right);
}

// 每个轮子独立 PID 速度闭环
fn wheel_pid(target: f64, actual: f64, state: &mut PidState) -> f64 {
    let error = target - actual;
    state.integral += error * DT;
    let derivative = (error - state.last_error) / DT;
    state.last_error = error;
    KP * error + KI * state.integral + KD * derivative
}"#;

const MECANUM_EXAMPLE_EN: &str = r#"// Mecanum Wheel Control Example (4-wheel omnidirectional)
// X-configuration: rollers at 45° to wheel axis
//
// Inverse Kinematics:
//   w_fl = (vx - vy - omega*K) / R
//   w_fr = (vx + vy + omega*K) / R
//   w_rl = (vx + vy - omega*K) / R
//   w_rr = (vx - vy + omega*K) / R
// where K = (L + W) / 2, L=wheelbase, W=trackwidth

fn control_loop(vx: f64, vy: f64, omega: f64) {
    let r = 0.076;  // wheel radius (m)
    let k = 0.375;  // (wheelbase + trackwidth) / 2

    let w = [
        (vx - vy - omega * k) / r,  // front-left
        (vx + vy + omega * k) / r,  // front-right
        (vx + vy - omega * k) / r,  // rear-left
        (vx - vy + omega * k) / r,  // rear-right
    ];

    for (i, &speed) in w.iter().enumerate() {
        set_motor_speed(i, speed);
    }
}

// Typical application: RoboMaster robot
// - Strafe and rotate simultaneously
// - PID per wheel + feedforward compensation"#;

const MECANUM_EXAMPLE_ZH: &str = r#"// 麦克纳姆轮底盘控制示例（四轮全向）
// X型布局: 滚子与轮轴成45°
//
// 逆运动学:
//   w_fl = (vx - vy - omega*K) / R
//   w_fr = (vx + vy + omega*K) / R
//   w_rl = (vx + vy - omega*K) / R
//   w_rr = (vx - vy + omega*K) / R
// K = (轴距 + 轮距) / 2

fn control_loop(vx: f64, vy: f64, omega: f64) {
    let r = 0.076;  // 轮半径 (m)
    let k = 0.375;  // (轴距 + 轮距) / 2

    let w = [
        (vx - vy - omega * k) / r,  // 左前
        (vx + vy + omega * k) / r,  // 右前
        (vx + vy - omega * k) / r,  // 左后
        (vx - vy + omega * k) / r,  // 右后
    ];

    for (i, &speed) in w.iter().enumerate() {
        set_motor_speed(i, speed);
    }
}

// 典型应用: RoboMaster 步兵机器人
// - 可同时平移和旋转
// - 每轮独立 PID + 前馈补偿"#;

const OMNI3_EXAMPLE_EN: &str = r#"// 3-Wheel Omni Drive Control Example
// 120° wheel arrangement
//
// Wheel 1: front (0°)
// Wheel 2: rear-left (120°)
// Wheel 3: rear-right (240°)
//
// Inverse Kinematics:
//   w1 = (-vy + omega*L) / R
//   w2 = (vx*sin60 + vy*cos60 + omega*L) / R
//   w3 = (-vx*sin60 + vy*cos60 + omega*L) / R

fn control_loop(vx: f64, vy: f64, omega: f64) {
    let r = 0.05;   // wheel radius
    let l = 0.15;   // chassis radius
    let sin60 = 0.866;
    let cos60 = 0.5;

    let w1 = (-vy + omega * l) / r;
    let w2 = (vx * sin60 + vy * cos60 + omega * l) / r;
    let w3 = (-vx * sin60 + vy * cos60 + omega * l) / r;

    set_motor_speed(0, w1);
    set_motor_speed(1, w2);
    set_motor_speed(2, w3);
}"#;

const OMNI3_EXAMPLE_ZH: &str = r#"// 三轮全向底盘控制示例
// 120° 轮组排列
//
// 轮 1: 前方 (0°)
// 轮 2: 左后 (120°)
// 轮 3: 右后 (240°)
//
// 逆运动学:
//   w1 = (-vy + omega*L) / R
//   w2 = (vx*sin60 + vy*cos60 + omega*L) / R
//   w3 = (-vx*sin60 + vy*cos60 + omega*L) / R

fn control_loop(vx: f64, vy: f64, omega: f64) {
    let r = 0.05;   // 轮半径
    let l = 0.15;   // 底盘半径
    let sin60 = 0.866;
    let cos60 = 0.5;

    let w1 = (-vy + omega * l) / r;
    let w2 = (vx * sin60 + vy * cos60 + omega * l) / r;
    let w3 = (-vx * sin60 + vy * cos60 + omega * l) / r;

    set_motor_speed(0, w1);
    set_motor_speed(1, w2);
    set_motor_speed(2, w3);
}"#;

const OMNI4_EXAMPLE_EN: &str = r#"// 4-Wheel Omni Drive Control Example
// 90° wheel arrangement (diamond/square)
//
// Inverse Kinematics (45° mounted):
//   w_fl = (-vx/√2 + vy/√2 + omega*L) / R
//   w_fr = ( vx/√2 + vy/√2 + omega*L) / R
//   w_rl = (-vx/√2 - vy/√2 + omega*L) / R
//   w_rr = ( vx/√2 - vy/√2 + omega*L) / R

fn control_loop(vx: f64, vy: f64, omega: f64) {
    let r = 0.05;
    let l = 0.15;
    let s = std::f64::consts::FRAC_1_SQRT_2;

    let w = [
        (-vx*s + vy*s + omega*l) / r,
        ( vx*s + vy*s + omega*l) / r,
        (-vx*s - vy*s + omega*l) / r,
        ( vx*s - vy*s + omega*l) / r,
    ];
    for (i, &speed) in w.iter().enumerate() {
        set_motor_speed(i, speed);
    }
}"#;

const OMNI4_EXAMPLE_ZH: &str = r#"// 四轮全向底盘控制示例
// 90° 轮组排列 (菱形/方形)
//
// 逆运动学 (45°安装):
//   w_fl = (-vx/√2 + vy/√2 + omega*L) / R
//   w_fr = ( vx/√2 + vy/√2 + omega*L) / R
//   w_rl = (-vx/√2 - vy/√2 + omega*L) / R
//   w_rr = ( vx/√2 - vy/√2 + omega*L) / R

fn control_loop(vx: f64, vy: f64, omega: f64) {
    let r = 0.05;
    let l = 0.15;
    let s = std::f64::consts::FRAC_1_SQRT_2;

    let w = [
        (-vx*s + vy*s + omega*l) / r,
        ( vx*s + vy*s + omega*l) / r,
        (-vx*s - vy*s + omega*l) / r,
        ( vx*s - vy*s + omega*l) / r,
    ];
    for (i, &speed) in w.iter().enumerate() {
        set_motor_speed(i, speed);
    }
}"#;

const ACKERMANN_EXAMPLE_EN: &str = r#"// Ackermann Steering Control Example
// Car-like robot with front steering, rear drive
//
// Geometry:
//   turn_radius = L / tan(steering_angle)
//   omega = vx / turn_radius
//
// Inner/outer wheel steering (Ackermann):
//   steer_left  = atan(L / (R - W/2))
//   steer_right = atan(L / (R + W/2))

fn control_loop(vx: f64, steering: f64) {
    let l = 0.3;    // wheelbase (m)
    let w = 0.2;    // track width (m)
    let r_wheel = 0.05;

    if steering.abs() < 0.001 {
        // Straight line
        let w_speed = vx / r_wheel;
        set_rear_motors(w_speed, w_speed);
        set_steering(0.0, 0.0);
        return;
    }

    let r_turn = l / steering.tan();
    let w_l = vx * (1.0 - w/(2.0*r_turn)) / r_wheel;
    let w_r = vx * (1.0 + w/(2.0*r_turn)) / r_wheel;

    let steer_l = (l / (r_turn - w/2.0)).atan();
    let steer_r = (l / (r_turn + w/2.0)).atan();

    set_rear_motors(w_l, w_r);
    set_steering(steer_l, steer_r);
}"#;

const ACKERMANN_EXAMPLE_ZH: &str = r#"// 阿克曼转向底盘控制示例
// 类汽车结构：前轮转向、后轮驱动
//
// 几何关系:
//   turn_radius = L / tan(steering_angle)
//   omega = vx / turn_radius
//
// 内外轮转角差 (阿克曼几何):
//   steer_left  = atan(L / (R - W/2))
//   steer_right = atan(L / (R + W/2))

fn control_loop(vx: f64, steering: f64) {
    let l = 0.3;    // 轴距 (m)
    let w = 0.2;    // 轮距 (m)
    let r_wheel = 0.05;

    if steering.abs() < 0.001 {
        // 直行
        let w_speed = vx / r_wheel;
        set_rear_motors(w_speed, w_speed);
        set_steering(0.0, 0.0);
        return;
    }

    let r_turn = l / steering.tan();
    let w_l = vx * (1.0 - w/(2.0*r_turn)) / r_wheel;
    let w_r = vx * (1.0 + w/(2.0*r_turn)) / r_wheel;

    let steer_l = (l / (r_turn - w/2.0)).atan();
    let steer_r = (l / (r_turn + w/2.0)).atan();

    set_rear_motors(w_l, w_r);
    set_steering(steer_l, steer_r);
}"#;

const TRACKED_EXAMPLE_EN: &str = r#"// Tracked (Tank) Drive Control Example
// Same kinematics as differential, without wheel radius
//
// Left track speed  = vx - omega * D
// Right track speed = vx + omega * D
//
// Skid steering: relies on track slip for turning

fn control_loop(vx: f64, omega: f64) {
    let d = 0.2; // half track width (m)

    let v_left  = vx - omega * d;
    let v_right = vx + omega * d;

    // Convert to PWM (0-255 range)
    let pwm_left  = (v_left  / MAX_SPEED * 255.0) as i16;
    let pwm_right = (v_right / MAX_SPEED * 255.0) as i16;

    set_track_pwm(LEFT_TRACK,  pwm_left);
    set_track_pwm(RIGHT_TRACK, pwm_right);
}"#;

const TRACKED_EXAMPLE_ZH: &str = r#"// 履带式底盘控制示例
// 运动学与差速驱动相同，无轮半径概念
//
// 左履带速度  = vx - omega * D
// 右履带速度  = vx + omega * D
//
// 差速转向：依靠履带滑动实现转弯

fn control_loop(vx: f64, omega: f64) {
    let d = 0.2; // 半履带宽度 (m)

    let v_left  = vx - omega * d;
    let v_right = vx + omega * d;

    // 转换为 PWM 信号 (0-255)
    let pwm_left  = (v_left  / MAX_SPEED * 255.0) as i16;
    let pwm_right = (v_right / MAX_SPEED * 255.0) as i16;

    set_track_pwm(LEFT_TRACK,  pwm_left);
    set_track_pwm(RIGHT_TRACK, pwm_right);
}"#;

const GENERIC_EXAMPLE: &str = r#"// Generic Robot Control Framework
//
// 1. Read sensors / encoders
// 2. Compute kinematics
// 3. Apply control algorithm (PID/LQR/MPC...)
// 4. Output to actuators
// 5. Loop at fixed rate

fn main_control_loop() {
    loop {
        let sensors = read_sensors();
        let target  = get_target_velocity();
        let wheels  = inverse_kinematics(target);

        for (motor, &target_w) in motors.iter().zip(wheels.iter()) {
            let actual_w = motor.read_encoder();
            let output   = pid_compute(target_w, actual_w);
            motor.set_pwm(output);
        }

        sleep(CONTROL_PERIOD);
    }
}"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_differential_inverse_straight() {
        let ws = ChassisKinematics::differential_inverse(100.0, 0.0, 50.0, 200.0);
        assert_eq!(ws.speeds.len(), 2);
        // 直行: 左右轮相同
        assert!((ws.speeds[0] - ws.speeds[1]).abs() < 1e-6);
    }

    #[test]
    fn test_differential_inverse_turn() {
        let ws = ChassisKinematics::differential_inverse(0.0, 1.0, 50.0, 200.0);
        // 原地转: 左右轮方向相反
        assert!(ws.speeds[0] < 0.0);
        assert!(ws.speeds[1] > 0.0);
    }

    #[test]
    fn test_differential_forward_roundtrip() {
        let ws = ChassisKinematics::differential_inverse(200.0, 0.5, 50.0, 250.0);
        let vel = ChassisKinematics::differential_forward(ws.speeds[0], ws.speeds[1], 50.0, 250.0);
        assert!((vel.vx - 200.0).abs() < 1.0);
        assert!((vel.omega - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_mecanum_inverse_strafe() {
        let ws = ChassisKinematics::mecanum_inverse(0.0, 100.0, 0.0, 76.0, 400.0, 350.0);
        assert_eq!(ws.speeds.len(), 4);
    }

    #[test]
    fn test_mecanum_forward_roundtrip() {
        let ws = ChassisKinematics::mecanum_inverse(100.0, 50.0, 0.3, 76.0, 400.0, 350.0);
        let vel = ChassisKinematics::mecanum_forward(
            ws.speeds[0],
            ws.speeds[1],
            ws.speeds[2],
            ws.speeds[3],
            76.0,
            400.0,
            350.0,
        );
        assert!((vel.vx - 100.0).abs() < 1.0);
        assert!((vel.vy - 50.0).abs() < 1.0);
    }

    #[test]
    fn test_omni3_inverse() {
        let ws = ChassisKinematics::omni3_inverse(100.0, 50.0, 0.0, 50.0, 150.0);
        assert_eq!(ws.speeds.len(), 3);
    }

    #[test]
    fn test_omni3_forward_roundtrip() {
        let ws = ChassisKinematics::omni3_inverse(100.0, 0.0, 0.5, 50.0, 150.0);
        let vel =
            ChassisKinematics::omni3_forward(ws.speeds[0], ws.speeds[1], ws.speeds[2], 50.0, 150.0);
        assert!((vel.vx - 100.0).abs() < 2.0);
        assert!((vel.omega - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_omni4_inverse() {
        let ws = ChassisKinematics::omni4_inverse(100.0, 50.0, 0.3, 50.0, 150.0);
        assert_eq!(ws.speeds.len(), 4);
    }

    #[test]
    fn test_ackermann_straight() {
        let (ws, sl, sr) = ChassisKinematics::ackermann_inverse(100.0, 0.0, 50.0, 300.0, 250.0);
        assert_eq!(ws.speeds.len(), 4);
        assert!((sl).abs() < 1e-6);
        assert!((sr).abs() < 1e-6);
    }

    #[test]
    fn test_ackermann_turn() {
        let (ws, sl, sr) = ChassisKinematics::ackermann_inverse(100.0, 0.3, 50.0, 300.0, 250.0);
        assert_eq!(ws.speeds.len(), 2);
        // 阿克曼几何: 内轮转角大于外轮
        assert!(sl.abs() > sr.abs());
    }

    #[test]
    fn test_tracked_inverse() {
        let ws = ChassisKinematics::tracked_inverse(100.0, 0.5, 300.0);
        assert_eq!(ws.speeds.len(), 2);
        assert!(ws.speeds[0] < ws.speeds[1]); // 左转时右侧更快
    }

    #[test]
    fn test_code_examples_exist() {
        for key in ChassisCodeExamples::all_chassis_keys() {
            let en = ChassisCodeExamples::get_example(key, "en");
            let zh = ChassisCodeExamples::get_example(key, "zh");
            assert!(!en.is_empty(), "Missing EN example for {}", key);
            assert!(!zh.is_empty(), "Missing ZH example for {}", key);
        }
    }

    #[test]
    fn test_chassis_velocity_default() {
        let v = ChassisVelocity::default();
        assert_eq!(v.vx, 0.0);
        assert_eq!(v.vy, 0.0);
        assert_eq!(v.omega, 0.0);
    }
}
