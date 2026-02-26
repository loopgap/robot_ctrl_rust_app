use crate::app::AppState;
use crate::i18n::Tr;
use crate::models::{ActuatorType, ChassisType};
use crate::views::ui_kit::{page_header, settings_card};
use egui::{self, Color32, RichText, ScrollArea, Ui};

pub fn show(ui: &mut Ui, state: &mut AppState) {
    let lang = state.lang();
    page_header(ui, Tr::tab_topology(lang), "topology");

    // ─── 预置拓扑 ────────────────────────────────────────
    settings_card(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 10.0;
            ui.label(RichText::new(format!("{}:", Tr::presets(lang))).strong());
            let presets = state.builtin_topologies.clone();
            for preset in &presets {
                if ui.button(&preset.name).clicked() {
                    state.topology = preset.clone();
                }
            }
        });
    });

    ui.add_space(10.0);

    // ─── 底盘类型选择 ─────────────────────────────────────
    settings_card(ui, |ui| {
        ui.label(RichText::new(Tr::chassis_type(lang)).size(15.0).strong());
        ui.add_space(8.0);

        let current_chassis = state.topology.chassis_type;
        let combo_w = ui.available_width().clamp(180.0, 320.0);
        egui::ComboBox::from_id_salt("chassis_type_combo")
            .selected_text(format!("{}", current_chassis))
            .width(combo_w)
            .show_ui(ui, |ui| {
                for &ct in ChassisType::all() {
                    if ui
                        .selectable_value(&mut state.topology.chassis_type, ct, format!("{}", ct))
                        .clicked()
                    {
                        let new_ct = ct;
                        state.topology.set_chassis_type(new_ct);
                    }
                }
            });

        ui.add_space(4.0);
        ui.label(
            RichText::new(state.topology.chassis_type.description())
                .size(12.0)
                .color(Color32::GRAY)
                .italics(),
        );

        ui.add_space(12.0);

        // ─── 几何参数 ────────────────────────────────────────
        ui.label(RichText::new(Tr::geometry_params(lang)).size(15.0).strong());
        ui.add_space(8.0);

        egui::Grid::new("geometry_grid")
            .num_columns(2)
            .spacing([20.0, 8.0])
            .show(ui, |ui| {
                ui.label(format!("{}:", Tr::name(lang)));
                ui.text_edit_singleline(&mut state.topology.name);
                ui.end_row();

                ui.label(format!("{}:", Tr::wheel_radius(lang)));
                ui.add(
                    egui::DragValue::new(&mut state.topology.wheel_radius)
                        .range(10.0..=500.0)
                        .speed(1.0),
                );
                ui.end_row();

                ui.label(format!("{}:", Tr::wheel_base(lang)));
                ui.add(
                    egui::DragValue::new(&mut state.topology.wheel_base)
                        .range(50.0..=2000.0)
                        .speed(1.0),
                );
                ui.end_row();

                ui.label(format!("{}:", Tr::track_width(lang)));
                ui.add(
                    egui::DragValue::new(&mut state.topology.track_width)
                        .range(50.0..=2000.0)
                        .speed(1.0),
                );
                ui.end_row();

                ui.label(format!("{}:", Tr::max_linear_vel(lang)));
                ui.add(
                    egui::DragValue::new(&mut state.topology.max_linear_vel)
                        .range(0.0..=10000.0)
                        .speed(10.0),
                );
                ui.end_row();

                ui.label(format!("{}:", Tr::max_angular_vel(lang)));
                ui.add(
                    egui::DragValue::new(&mut state.topology.max_angular_vel)
                        .range(0.0..=20.0)
                        .speed(0.1),
                );
            });
    });

    ui.add_space(10.0);

    // ─── 电机/关节配置 ────────────────────────────────────
    settings_card(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new(format!(
                    "{} ({})",
                    Tr::motors_joints(lang),
                    state.topology.motors.len()
                ))
                .size(15.0)
                .strong(),
            );
            ui.add_space(12.0);
            if ui.button(Tr::add_motor(lang)).clicked() {
                let id = state.topology.motors.len() as u8;
                let mc = crate::models::robot_topology::MotorConfig {
                    id,
                    name: format!("Motor_{}", id + 1),
                    ..Default::default()
                };
                state.topology.motors.push(mc);
            }
        });
        ui.add_space(8.0);

        let mut remove_motor: Option<usize> = None;

        ScrollArea::vertical().max_height(280.0).show(ui, |ui| {
            for mi in 0..state.topology.motors.len() {
                egui::Frame::new()
                    .fill(Color32::from_rgba_premultiplied(35, 40, 55, 200))
                    .corner_radius(6.0)
                    .inner_margin(10.0)
                    .outer_margin(egui::Margin::symmetric(0, 3))
                    .show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing.x = 8.0;
                            ui.checkbox(&mut state.topology.motors[mi].enabled, "");
                            ui.label(
                                RichText::new(format!("#{}", state.topology.motors[mi].id))
                                    .strong(),
                            );
                            ui.add(
                                egui::TextEdit::singleline(&mut state.topology.motors[mi].name)
                                    .desired_width(110.0),
                            );

                            let at = state.topology.motors[mi].actuator_type;
                            egui::ComboBox::from_id_salt(format!("motor_type_{}", mi))
                                .selected_text(format!("{}", at))
                                .width(130.0)
                                .show_ui(ui, |ui| {
                                    for &a in ActuatorType::all() {
                                        ui.selectable_value(
                                            &mut state.topology.motors[mi].actuator_type,
                                            a,
                                            format!("{}", a),
                                        );
                                    }
                                });

                            ui.checkbox(&mut state.topology.motors[mi].reversed, "Rev");
                        });

                        ui.add_space(4.0);

                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing.x = 10.0;
                            ui.label("RPM:");
                            ui.add(
                                egui::DragValue::new(&mut state.topology.motors[mi].max_rpm)
                                    .range(0.0..=50000.0)
                                    .speed(10.0),
                            );
                            ui.label("Max A:");
                            ui.add(
                                egui::DragValue::new(&mut state.topology.motors[mi].max_current)
                                    .range(0.0..=100.0)
                                    .speed(0.1),
                            );
                            ui.label("Gear:");
                            ui.add(
                                egui::DragValue::new(&mut state.topology.motors[mi].gear_ratio)
                                    .range(0.01..=1000.0)
                                    .speed(0.1),
                            );
                            ui.label("PPR:");
                            ui.add(
                                egui::DragValue::new(&mut state.topology.motors[mi].encoder_ppr)
                                    .range(1..=65535),
                            );

                            if ui.button("Remove").clicked() {
                                remove_motor = Some(mi);
                            }
                        });
                    });
            }
        });

        if let Some(ri) = remove_motor {
            if state.topology.motors.len() > 1 {
                state.topology.motors.remove(ri);
                for (i, m) in state.topology.motors.iter_mut().enumerate() {
                    m.id = i as u8;
                }
            }
        }
    });

    ui.add_space(10.0);

    // ─── 拓扑可视化 ───────────────────────────────────────
    settings_card(ui, |ui| {
        ui.label(RichText::new(Tr::topology_viz(lang)).size(15.0).strong());
        ui.add_space(8.0);
        let art = chassis_ascii_art(state.topology.chassis_type);
        ui.label(
            RichText::new(art)
                .size(12.0)
                .monospace()
                .color(Color32::from_rgb(0, 200, 255)),
        );
    });
}

fn chassis_ascii_art(ct: ChassisType) -> &'static str {
    match ct {
        ChassisType::Differential => {
            r#"
    [M1]──┐     ┌──[M2]
     O    ├─────┤    O
          │ROBOT│
          └─────┘
      Left        Right
"#
        }
        ChassisType::Mecanum => {
            r#"
    [M1]╲  ┌─────┐  ╱[M2]
          ├─     ─┤
          │ ROBOT │
          ├─     ─┤
    [M3]╱  └─────┘  ╲[M4]
      FL     Body     FR
"#
        }
        ChassisType::Omni3 => {
            r#"
          [M1]
           ◯
          / \
         /   \
        /ROBOT\
   [M2]◯───────◯[M3]
"#
        }
        ChassisType::Ackermann => {
            r#"
    [Steer]─┐     ┌─[Steer]
      ◯     ├─────┤     ◯
            │ROBOT│
     [M1]◯  ├─────┤  ◯[M2]
            Drive Axle
"#
        }
        ChassisType::Tracked => {
            r#"
   ╔═══════════╗
   ║[M1]  ROBOT║[M2]
   ║ ████████  ║
   ║ ████████  ║
   ╚═══════════╝
   Left Track  Right
"#
        }
        ChassisType::SixDofArm => {
            r#"
         [J6]──╮ End Effector
     [J5]──╯   │
    [J4]──╯    │
   [J3]──╯     │
  [J2]──╯      ┃
 [J1]═╧═══ Base
"#
        }
        ChassisType::Scara => {
            r#"
        [J4]Z↕ End
    ┌───╨───┐
   [J3]     │
    ├───────┘
   [J2]
    │
   [J1]══Base
"#
        }
        ChassisType::DeltaRobot => {
            r#"
   [M1]     [M2]     [M3]
    ╲         │        ╱
     ╲        │       ╱
      ╲       │      ╱
       ╲──End Eff──╱
"#
        }
        _ => {
            r#"
   ┌─────────────┐
   │   CUSTOM    │
   │   ROBOT     │
   │   CONFIG    │
   └─────────────┘
"#
        }
    }
}
