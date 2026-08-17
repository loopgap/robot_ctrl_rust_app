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
    fn test_code_examples_exist() {
        for key in ChassisCodeExamples::all_chassis_keys() {
            let en = ChassisCodeExamples::get_example(key, "en");
            let zh = ChassisCodeExamples::get_example(key, "zh");
            assert!(!en.is_empty(), "Missing EN example for {}", key);
            assert!(!zh.is_empty(), "Missing ZH example for {}", key);
        }
    }

    #[test]
    fn test_all_chassis_keys_count() {
        assert_eq!(ChassisCodeExamples::all_chassis_keys().len(), 6);
    }

    #[test]
    fn test_unknown_chassis_returns_generic() {
        let example = ChassisCodeExamples::get_example("UnknownType", "en");
        assert_eq!(example, GENERIC_EXAMPLE);
        assert!(!example.is_empty());
    }

    #[test]
    fn test_unknown_lang_returns_generic() {
        let example = ChassisCodeExamples::get_example("Differential", "fr");
        assert_eq!(example, GENERIC_EXAMPLE);
    }

    #[test]
    fn test_examples_contain_code_signatures() {
        // EN examples should contain Rust code keywords
        for key in ChassisCodeExamples::all_chassis_keys() {
            let en = ChassisCodeExamples::get_example(key, "en");
            assert!(
                en.contains("fn ") || en.contains("struct ") || en.contains("let "),
                "EN example for {} should contain Rust code",
                key
            );
        }
    }

    #[test]
    fn test_each_chassis_has_unique_content() {
        // Each chassis type should have distinct example content
        let examples: Vec<&str> = ChassisCodeExamples::all_chassis_keys()
            .iter()
            .map(|k| ChassisCodeExamples::get_example(k, "en"))
            .collect();
        for i in 0..examples.len() {
            for j in (i + 1)..examples.len() {
                assert_ne!(
                    examples[i],
                    examples[j],
                    "Examples for chassis {} and {} should differ",
                    ChassisCodeExamples::all_chassis_keys()[i],
                    ChassisCodeExamples::all_chassis_keys()[j]
                );
            }
        }
    }

    #[test]
    fn test_zh_and_en_differ() {
        // Chinese and English examples should be different translations
        for key in ChassisCodeExamples::all_chassis_keys() {
            let en = ChassisCodeExamples::get_example(key, "en");
            let zh = ChassisCodeExamples::get_example(key, "zh");
            assert_ne!(en, zh, "EN and ZH examples for {} should differ", key);
        }
    }
}
