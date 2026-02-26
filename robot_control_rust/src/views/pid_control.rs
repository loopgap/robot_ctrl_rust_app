use crate::app::AppState;
use crate::i18n::Tr;
use crate::models::{ChassisCodeExamples, ControlAlgorithmType};
use crate::views::ui_kit::{page_header, settings_card};
use egui::{self, Color32, RichText, Ui};

const PARAM_LABEL_WIDTH: f32 = 130.0;
const PARAM_INPUT_WIDTH: f32 = 96.0;

pub fn show(ui: &mut Ui, state: &mut AppState) {
    let lang = state.lang();
    page_header(ui, Tr::tab_pid_control(lang), "pid");

    settings_card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 12.0;

            let run_btn = if state.is_running {
                RichText::new(Tr::stop_control(lang))
                    .size(15.0)
                    .color(Color32::from_rgb(255, 100, 100))
            } else {
                RichText::new(Tr::start_control(lang))
                    .size(15.0)
                    .color(Color32::from_rgb(100, 255, 100))
            };
            if ui.button(run_btn).clicked() {
                state.toggle_running();
            }

            if ui
                .button(
                    RichText::new(Tr::emergency_stop(lang))
                        .size(15.0)
                        .color(Color32::RED)
                        .strong(),
                )
                .clicked()
            {
                state.emergency_stop();
            }

            ui.add_space(12.0);

            if state.is_running {
                ui.label(
                    RichText::new(Tr::running(lang))
                        .color(Color32::from_rgb(46, 160, 67))
                        .strong(),
                );
            } else {
                ui.label(RichText::new(Tr::stopped(lang)).color(Color32::GRAY));
            }
        });
    });

    ui.add_space(10.0);

    settings_card(ui, |ui| {
        ui.label(
            RichText::new(Tr::algorithm_select(lang))
                .size(16.0)
                .strong(),
        );
        ui.add_space(8.0);

        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(10.0, 8.0);
            for alg in ControlAlgorithmType::all() {
                let name = match lang {
                    crate::i18n::Language::English => alg.name_en(),
                    crate::i18n::Language::Chinese => alg.name_zh(),
                };
                let selected = state.active_algorithm == *alg;
                let btn = if selected {
                    RichText::new(name.to_string())
                        .strong()
                        .color(Color32::from_rgb(88, 166, 255))
                } else {
                    RichText::new(name.to_string())
                };
                if ui.button(btn).clicked() && !state.is_running {
                    state.active_algorithm = *alg;
                }
            }
        });

        let desc = match lang {
            crate::i18n::Language::English => state.active_algorithm.desc_en(),
            crate::i18n::Language::Chinese => state.active_algorithm.desc_zh(),
        };
        ui.add_space(6.0);
        ui.label(RichText::new(desc).italics().color(Color32::GRAY));
    });

    ui.add_space(10.0);

    settings_card(ui, |ui| match state.active_algorithm {
        ControlAlgorithmType::ClassicPid => show_classic_pid(ui, state),
        ControlAlgorithmType::IncrementalPid => show_incremental_pid(ui, state),
        ControlAlgorithmType::BangBang => show_bang_bang(ui, state),
        ControlAlgorithmType::FuzzyPid => show_fuzzy_pid(ui, state),
        ControlAlgorithmType::CascadePid => show_cascade_pid(ui, state),
        ControlAlgorithmType::SmithPredictor => show_smith_predictor(ui, state),
        ControlAlgorithmType::Adrc => show_adrc(ui, state),
        ControlAlgorithmType::Ladrc => show_ladrc(ui, state),
        ControlAlgorithmType::Lqr => show_lqr(ui, state),
        ControlAlgorithmType::Mpc => show_mpc(ui, state),
    });

    // ─── 预设管理（仅经典PID有预设） ─────────────────────
    if state.active_algorithm == ControlAlgorithmType::ClassicPid {
        ui.add_space(10.0);
        settings_card(ui, |ui| {
            ui.label(RichText::new(Tr::presets(lang)).size(16.0).strong());
            ui.add_space(8.0);

            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(10.0, 8.0);
                let presets = state.presets.clone();
                for preset in &presets {
                    if ui
                        .button(preset.name.to_string())
                        .on_hover_text(&preset.description)
                        .clicked()
                    {
                        preset.apply_to(&mut state.pid);
                        state.ui.kp_text = format!("{:.3}", state.pid.kp);
                        state.ui.ki_text = format!("{:.3}", state.pid.ki);
                        state.ui.kd_text = format!("{:.3}", state.pid.kd);
                        state.ui.setpoint_text = format!("{:.3}", state.pid.setpoint);
                        state.ui.output_limit_text = format!("{:.1}", state.pid.output_limit);
                        state.ui.integral_limit_text = format!("{:.1}", state.pid.integral_limit);
                        state.status_message = Tr::applied_preset(&preset.name, lang);
                    }
                }
            });

            ui.add_space(8.0);

            ui.collapsing(Tr::save_preset(lang), |ui| {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(format!("{}:", Tr::name(lang)));
                    ui.text_edit_singleline(&mut state.ui.preset_name);
                });
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(format!("{}:", Tr::description(lang)));
                    ui.text_edit_singleline(&mut state.ui.preset_desc);
                });
                ui.add_space(6.0);
                if ui.button(Tr::save(lang)).clicked() && !state.ui.preset_name.is_empty() {
                    let p = crate::models::Preset::from_controller(
                        state.ui.preset_name.clone(),
                        state.ui.preset_desc.clone(),
                        &state.pid,
                    );
                    state.presets.push(p);
                    state.status_message = Tr::applied_preset(&state.ui.preset_name, lang);
                    state.ui.preset_name.clear();
                    state.ui.preset_desc.clear();
                }
            });
        });
    }

    // ─── 当前状态 ────────────────────────────────────────
    ui.add_space(10.0);
    settings_card(ui, |ui| {
        ui.label(RichText::new(Tr::current_state(lang)).size(16.0).strong());
        ui.add_space(8.0);

        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(16.0, 8.0);
            ui.label(format!("{}:", Tr::position(lang)));
            ui.label(
                RichText::new(format!("{:.3}", state.current_state.position))
                    .strong()
                    .monospace(),
            );
            ui.label(format!("{}:", Tr::error_ch(lang)));
            ui.label(
                RichText::new(format!("{:.3}", state.current_state.error))
                    .strong()
                    .monospace(),
            );
            ui.label(format!("{}:", Tr::velocity(lang)));
            ui.label(
                RichText::new(format!("{:.3}", state.current_state.velocity))
                    .strong()
                    .monospace(),
            );
            ui.label(format!("{} :", Tr::pid_output(lang)));
            ui.label(
                RichText::new(format!("{:.3}", state.current_state.pid_output))
                    .strong()
                    .monospace(),
            );
        });

        ui.add_space(8.0);

        if ui.button(Tr::reset(lang)).clicked() {
            state.reset_active_algorithm();
            state.status_message = format!("{} reset", state.active_algorithm.name_en());
        }
    });

    // ─── 底盘运动学代码示例 ──────────────────────────────
    ui.add_space(10.0);
    settings_card(ui, |ui| {
        ui.collapsing(
            RichText::new(Tr::chassis_kinematics(lang))
                .size(16.0)
                .strong(),
            |ui| {
                ui.add_space(4.0);
                ui.label(
                    RichText::new(Tr::chassis_kinematics_desc(lang))
                        .italics()
                        .color(Color32::GRAY),
                );
                ui.add_space(8.0);

                let lang_key = match lang {
                    crate::i18n::Language::English => "en",
                    crate::i18n::Language::Chinese => "zh",
                };

                for &key in ChassisCodeExamples::all_chassis_keys() {
                    ui.collapsing(key.to_string(), |ui| {
                        let code = ChassisCodeExamples::get_example(key, lang_key);
                        egui::ScrollArea::vertical()
                            .max_height(300.0)
                            .show(ui, |ui| {
                                ui.add(
                                    egui::TextEdit::multiline(&mut code.to_string())
                                        .font(egui::TextStyle::Monospace)
                                        .desired_width(f32::INFINITY)
                                        .interactive(false),
                                );
                            });
                    });
                }
            },
        );
    });
}

// ═══════════════════════════════════════════════════════════════
// 各算法参数面板
// ═══════════════════════════════════════════════════════════════

fn show_classic_pid(ui: &mut Ui, state: &mut AppState) {
    let lang = state.lang();
    ui.label(RichText::new(Tr::pid_params(lang)).size(16.0).strong());
    ui.add_space(8.0);

    egui::Grid::new("pid_params_grid")
        .num_columns(3)
        .spacing([24.0, 12.0])
        .show(ui, |ui| {
            // Kp
            ui.add_sized(
                [PARAM_LABEL_WIDTH, 20.0],
                egui::Label::new(RichText::new("Kp:").strong()),
            );
            ui.add(egui::Slider::new(&mut state.pid.kp, 0.0..=10.0).step_by(0.001));
            ui.add(
                egui::TextEdit::singleline(&mut state.ui.kp_text).desired_width(PARAM_INPUT_WIDTH),
            );
            if let Ok(v) = state.ui.kp_text.parse::<f64>() {
                state.pid.kp = v;
            } else {
                state.ui.kp_text = format!("{:.3}", state.pid.kp);
            }
            ui.end_row();

            // Ki
            ui.add_sized(
                [PARAM_LABEL_WIDTH, 20.0],
                egui::Label::new(RichText::new("Ki:").strong()),
            );
            ui.add(egui::Slider::new(&mut state.pid.ki, 0.0..=5.0).step_by(0.001));
            ui.add(
                egui::TextEdit::singleline(&mut state.ui.ki_text).desired_width(PARAM_INPUT_WIDTH),
            );
            if let Ok(v) = state.ui.ki_text.parse::<f64>() {
                state.pid.ki = v;
            } else {
                state.ui.ki_text = format!("{:.3}", state.pid.ki);
            }
            ui.end_row();

            // Kd
            ui.add_sized(
                [PARAM_LABEL_WIDTH, 20.0],
                egui::Label::new(RichText::new("Kd:").strong()),
            );
            ui.add(egui::Slider::new(&mut state.pid.kd, 0.0..=2.0).step_by(0.001));
            ui.add(
                egui::TextEdit::singleline(&mut state.ui.kd_text).desired_width(PARAM_INPUT_WIDTH),
            );
            if let Ok(v) = state.ui.kd_text.parse::<f64>() {
                state.pid.kd = v;
            } else {
                state.ui.kd_text = format!("{:.3}", state.pid.kd);
            }
            ui.end_row();

            // Setpoint
            ui.add_sized(
                [PARAM_LABEL_WIDTH, 20.0],
                egui::Label::new(RichText::new("Setpoint:").strong()),
            );
            ui.add(egui::Slider::new(&mut state.pid.setpoint, -500.0..=500.0).step_by(0.1));
            ui.add(
                egui::TextEdit::singleline(&mut state.ui.setpoint_text)
                    .desired_width(PARAM_INPUT_WIDTH),
            );
            if let Ok(v) = state.ui.setpoint_text.parse::<f64>() {
                state.pid.setpoint = v;
            } else {
                state.ui.setpoint_text = format!("{:.3}", state.pid.setpoint);
            }
            ui.end_row();

            // Output Limit
            ui.add_sized([PARAM_LABEL_WIDTH, 20.0], egui::Label::new("Output Limit:"));
            ui.add(egui::Slider::new(&mut state.pid.output_limit, 1.0..=1000.0).step_by(1.0));
            ui.add(
                egui::TextEdit::singleline(&mut state.ui.output_limit_text)
                    .desired_width(PARAM_INPUT_WIDTH),
            );
            if let Ok(v) = state.ui.output_limit_text.parse::<f64>() {
                state.pid.output_limit = v;
            } else {
                state.ui.output_limit_text = format!("{:.1}", state.pid.output_limit);
            }
            ui.end_row();

            // Integral Limit
            ui.add_sized(
                [PARAM_LABEL_WIDTH, 20.0],
                egui::Label::new("Integral Limit:"),
            );
            ui.add(egui::Slider::new(&mut state.pid.integral_limit, 1.0..=1000.0).step_by(1.0));
            ui.add(
                egui::TextEdit::singleline(&mut state.ui.integral_limit_text)
                    .desired_width(PARAM_INPUT_WIDTH),
            );
            if let Ok(v) = state.ui.integral_limit_text.parse::<f64>() {
                state.pid.integral_limit = v;
            } else {
                state.ui.integral_limit_text = format!("{:.1}", state.pid.integral_limit);
            }
            ui.end_row();
        });

    ui.add_space(12.0);

    ui.collapsing(Tr::advanced_options(lang), |ui| {
        ui.add_space(6.0);
        egui::Grid::new("pid_advanced_grid")
            .num_columns(2)
            .spacing([20.0, 8.0])
            .show(ui, |ui| {
                ui.label(format!("{}:", Tr::deriv_filter(lang)));
                ui.add(
                    egui::Slider::new(&mut state.pid.derivative_filter, 0.0..=1.0).step_by(0.01),
                );
                ui.end_row();

                ui.checkbox(&mut state.pid.anti_windup, Tr::anti_windup(lang));
                ui.label("");
                ui.end_row();

                ui.label(format!("{}:", Tr::feedforward(lang)));
                ui.add(egui::Slider::new(&mut state.pid.feedforward, 0.0..=5.0).step_by(0.01));
                ui.end_row();

                ui.label(format!("{}:", Tr::dead_zone(lang)));
                ui.add(egui::Slider::new(&mut state.pid.dead_zone, 0.0..=10.0).step_by(0.1));
                ui.end_row();
            });
    });

    // PID 内部状态
    ui.add_space(8.0);
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(14.0, 8.0);
        ui.label("Integral:");
        ui.label(
            RichText::new(format!("{:.3}", state.pid.integral))
                .strong()
                .monospace(),
        );
        ui.label("Derivative:");
        ui.label(
            RichText::new(format!("{:.3}", state.pid.derivative))
                .strong()
                .monospace(),
        );
    });
}

fn show_incremental_pid(ui: &mut Ui, state: &mut AppState) {
    let lang = state.lang();
    ui.label(RichText::new(Tr::pid_params(lang)).size(16.0).strong());
    ui.add_space(8.0);

    egui::Grid::new("incr_pid_grid")
        .num_columns(2)
        .spacing([24.0, 12.0])
        .show(ui, |ui| {
            ui.add_sized(
                [PARAM_LABEL_WIDTH, 20.0],
                egui::Label::new(RichText::new("Kp:").strong()),
            );
            ui.add(egui::Slider::new(&mut state.incremental_pid.kp, 0.0..=10.0).step_by(0.001));
            ui.end_row();
            ui.add_sized(
                [PARAM_LABEL_WIDTH, 20.0],
                egui::Label::new(RichText::new("Ki:").strong()),
            );
            ui.add(egui::Slider::new(&mut state.incremental_pid.ki, 0.0..=5.0).step_by(0.001));
            ui.end_row();
            ui.add_sized(
                [PARAM_LABEL_WIDTH, 20.0],
                egui::Label::new(RichText::new("Kd:").strong()),
            );
            ui.add(egui::Slider::new(&mut state.incremental_pid.kd, 0.0..=2.0).step_by(0.001));
            ui.end_row();
            ui.add_sized(
                [PARAM_LABEL_WIDTH, 20.0],
                egui::Label::new(RichText::new("Setpoint:").strong()),
            );
            ui.add(
                egui::Slider::new(&mut state.incremental_pid.setpoint, -500.0..=500.0).step_by(0.1),
            );
            ui.end_row();
            ui.add_sized([PARAM_LABEL_WIDTH, 20.0], egui::Label::new("Output Limit:"));
            ui.add(
                egui::Slider::new(&mut state.incremental_pid.output_limit, 1.0..=1000.0)
                    .step_by(1.0),
            );
            ui.end_row();
            ui.add_sized(
                [PARAM_LABEL_WIDTH, 20.0],
                egui::Label::new(format!("{}:", Tr::increment_limit(lang))),
            );
            ui.add(
                egui::Slider::new(&mut state.incremental_pid.increment_limit, 0.1..=200.0)
                    .step_by(0.1),
            );
            ui.end_row();
            ui.add_sized(
                [PARAM_LABEL_WIDTH, 20.0],
                egui::Label::new(format!("{}:", Tr::dead_zone(lang))),
            );
            ui.add(
                egui::Slider::new(&mut state.incremental_pid.dead_zone, 0.0..=10.0).step_by(0.1),
            );
            ui.end_row();
            ui.add_sized(
                [PARAM_LABEL_WIDTH, 20.0],
                egui::Label::new(format!("{}:", Tr::output_ramp(lang))),
            );
            ui.add(
                egui::Slider::new(&mut state.incremental_pid.output_ramp, 0.0..=1000.0)
                    .step_by(1.0),
            );
            ui.end_row();
        });

    ui.add_space(8.0);
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(14.0, 8.0);
        ui.label(format!("{}:", Tr::last_increment(lang)));
        ui.label(
            RichText::new(format!("{:.4}", state.incremental_pid.last_increment))
                .strong()
                .monospace(),
        );
        ui.label("Output:");
        ui.label(
            RichText::new(format!("{:.3}", state.incremental_pid.output))
                .strong()
                .monospace(),
        );
    });
}

fn show_bang_bang(ui: &mut Ui, state: &mut AppState) {
    let lang = state.lang();
    ui.label(RichText::new("Bang-Bang").size(16.0).strong());
    ui.add_space(8.0);

    egui::Grid::new("bangbang_grid")
        .num_columns(2)
        .spacing([20.0, 10.0])
        .show(ui, |ui| {
            ui.label(RichText::new("Setpoint:").strong());
            ui.add(egui::Slider::new(&mut state.bang_bang.setpoint, -500.0..=500.0).step_by(0.1));
            ui.end_row();
            ui.label(format!("{}:", Tr::output_high(lang)));
            ui.add(egui::Slider::new(&mut state.bang_bang.output_high, 0.0..=1000.0).step_by(1.0));
            ui.end_row();
            ui.label(format!("{}:", Tr::output_low(lang)));
            ui.add(egui::Slider::new(&mut state.bang_bang.output_low, -1000.0..=0.0).step_by(1.0));
            ui.end_row();
            ui.label(format!("{}:", Tr::hysteresis(lang)));
            ui.add(egui::Slider::new(&mut state.bang_bang.hysteresis, 0.0..=50.0).step_by(0.1));
            ui.end_row();
            ui.label(format!("{}:", Tr::dead_band(lang)));
            ui.add(egui::Slider::new(&mut state.bang_bang.dead_band, 0.0..=20.0).step_by(0.1));
            ui.end_row();
        });

    ui.add_space(8.0);
    let state_str = match state.bang_bang.last_state {
        crate::models::bang_bang::BangBangState::Off => "OFF",
        crate::models::bang_bang::BangBangState::High => "HIGH \u{2B06}",
        crate::models::bang_bang::BangBangState::Low => "LOW \u{2B07}",
    };
    ui.horizontal(|ui| {
        ui.label(format!("{}:", Tr::switch_state(lang)));
        let color = match state.bang_bang.last_state {
            crate::models::bang_bang::BangBangState::Off => Color32::GRAY,
            crate::models::bang_bang::BangBangState::High => Color32::from_rgb(46, 160, 67),
            crate::models::bang_bang::BangBangState::Low => Color32::from_rgb(255, 100, 100),
        };
        ui.label(RichText::new(state_str).strong().color(color).monospace());
        ui.label("Output:");
        ui.label(
            RichText::new(format!("{:.1}", state.bang_bang.output))
                .strong()
                .monospace(),
        );
    });
}

fn show_fuzzy_pid(ui: &mut Ui, state: &mut AppState) {
    let lang = state.lang();

    ui.label(RichText::new(Tr::base_params(lang)).size(16.0).strong());
    ui.add_space(8.0);

    egui::Grid::new("fuzzy_base_grid")
        .num_columns(2)
        .spacing([20.0, 10.0])
        .show(ui, |ui| {
            ui.label(RichText::new("Kp (base):").strong());
            ui.add(egui::Slider::new(&mut state.fuzzy_pid.kp_base, 0.0..=10.0).step_by(0.001));
            ui.end_row();
            ui.label(RichText::new("Ki (base):").strong());
            ui.add(egui::Slider::new(&mut state.fuzzy_pid.ki_base, 0.0..=5.0).step_by(0.001));
            ui.end_row();
            ui.label(RichText::new("Kd (base):").strong());
            ui.add(egui::Slider::new(&mut state.fuzzy_pid.kd_base, 0.0..=2.0).step_by(0.001));
            ui.end_row();
            ui.label(RichText::new("Setpoint:").strong());
            ui.add(egui::Slider::new(&mut state.fuzzy_pid.setpoint, -500.0..=500.0).step_by(0.1));
            ui.end_row();
            ui.label("Output Limit:");
            ui.add(egui::Slider::new(&mut state.fuzzy_pid.output_limit, 1.0..=1000.0).step_by(1.0));
            ui.end_row();
            ui.label("Integral Limit:");
            ui.add(
                egui::Slider::new(&mut state.fuzzy_pid.integral_limit, 1.0..=1000.0).step_by(1.0),
            );
            ui.end_row();
        });

    ui.add_space(8.0);
    ui.collapsing(Tr::fuzzy_tuning_range(lang), |ui| {
        ui.add_space(6.0);
        egui::Grid::new("fuzzy_range_grid")
            .num_columns(2)
            .spacing([20.0, 8.0])
            .show(ui, |ui| {
                ui.label("\u{0394}Kp Range:");
                ui.add(egui::Slider::new(&mut state.fuzzy_pid.kp_range, 0.0..=5.0).step_by(0.01));
                ui.end_row();
                ui.label("\u{0394}Ki Range:");
                ui.add(egui::Slider::new(&mut state.fuzzy_pid.ki_range, 0.0..=2.0).step_by(0.01));
                ui.end_row();
                ui.label("\u{0394}Kd Range:");
                ui.add(egui::Slider::new(&mut state.fuzzy_pid.kd_range, 0.0..=1.0).step_by(0.01));
                ui.end_row();
                ui.label(format!("{}:", Tr::error_scale(lang)));
                ui.add(
                    egui::Slider::new(&mut state.fuzzy_pid.error_scale, 1.0..=100.0).step_by(0.5),
                );
                ui.end_row();
                ui.label(format!("{}:", Tr::ec_scale(lang)));
                ui.add(egui::Slider::new(&mut state.fuzzy_pid.ec_scale, 1.0..=100.0).step_by(0.5));
                ui.end_row();
            });
    });

    ui.add_space(8.0);
    ui.label(
        RichText::new(Tr::effective_params(lang))
            .size(14.0)
            .strong(),
    );
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(14.0, 8.0);
        ui.label("Kp_eff:");
        ui.label(
            RichText::new(format!("{:.4}", state.fuzzy_pid.effective_kp))
                .strong()
                .monospace(),
        );
        ui.label("Ki_eff:");
        ui.label(
            RichText::new(format!("{:.4}", state.fuzzy_pid.effective_ki))
                .strong()
                .monospace(),
        );
        ui.label("Kd_eff:");
        ui.label(
            RichText::new(format!("{:.4}", state.fuzzy_pid.effective_kd))
                .strong()
                .monospace(),
        );
    });
}

fn show_cascade_pid(ui: &mut Ui, state: &mut AppState) {
    let lang = state.lang();

    // 外环参数
    ui.label(RichText::new(Tr::outer_loop(lang)).size(16.0).strong());
    ui.add_space(8.0);
    egui::Grid::new("cascade_outer_grid")
        .num_columns(2)
        .spacing([20.0, 10.0])
        .show(ui, |ui| {
            ui.label(RichText::new("Kp:").strong());
            ui.add(egui::Slider::new(&mut state.cascade_pid.outer_kp, 0.0..=10.0).step_by(0.001));
            ui.end_row();
            ui.label(RichText::new("Ki:").strong());
            ui.add(egui::Slider::new(&mut state.cascade_pid.outer_ki, 0.0..=5.0).step_by(0.001));
            ui.end_row();
            ui.label(RichText::new("Kd:").strong());
            ui.add(egui::Slider::new(&mut state.cascade_pid.outer_kd, 0.0..=2.0).step_by(0.001));
            ui.end_row();
            ui.label("Output Limit:");
            ui.add(
                egui::Slider::new(&mut state.cascade_pid.outer_output_limit, 1.0..=500.0)
                    .step_by(1.0),
            );
            ui.end_row();
        });

    ui.add_space(8.0);

    // 内环参数
    ui.label(RichText::new(Tr::inner_loop(lang)).size(16.0).strong());
    ui.add_space(8.0);
    egui::Grid::new("cascade_inner_grid")
        .num_columns(2)
        .spacing([20.0, 10.0])
        .show(ui, |ui| {
            ui.label(RichText::new("Kp:").strong());
            ui.add(egui::Slider::new(&mut state.cascade_pid.inner_kp, 0.0..=10.0).step_by(0.001));
            ui.end_row();
            ui.label(RichText::new("Ki:").strong());
            ui.add(egui::Slider::new(&mut state.cascade_pid.inner_ki, 0.0..=5.0).step_by(0.001));
            ui.end_row();
            ui.label(RichText::new("Kd:").strong());
            ui.add(egui::Slider::new(&mut state.cascade_pid.inner_kd, 0.0..=2.0).step_by(0.001));
            ui.end_row();
            ui.label("Output Limit:");
            ui.add(
                egui::Slider::new(&mut state.cascade_pid.inner_output_limit, 1.0..=1000.0)
                    .step_by(1.0),
            );
            ui.end_row();
        });

    ui.add_space(4.0);
    ui.label(RichText::new("Setpoint:").strong());
    ui.add(egui::Slider::new(&mut state.cascade_pid.setpoint, -500.0..=500.0).step_by(0.1));

    ui.add_space(8.0);
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(14.0, 8.0);
        ui.label(format!("{}:", Tr::outer_output(lang)));
        ui.label(
            RichText::new(format!("{:.3}", state.cascade_pid.outer_output))
                .strong()
                .monospace(),
        );
        ui.label("Output:");
        ui.label(
            RichText::new(format!("{:.3}", state.cascade_pid.output))
                .strong()
                .monospace(),
        );
    });
}

fn show_smith_predictor(ui: &mut Ui, state: &mut AppState) {
    let lang = state.lang();

    ui.label(RichText::new(Tr::pid_params(lang)).size(16.0).strong());
    ui.add_space(8.0);

    egui::Grid::new("smith_pid_grid")
        .num_columns(2)
        .spacing([20.0, 10.0])
        .show(ui, |ui| {
            ui.label(RichText::new("Kp:").strong());
            ui.add(egui::Slider::new(&mut state.smith_predictor.kp, 0.0..=10.0).step_by(0.001));
            ui.end_row();
            ui.label(RichText::new("Ki:").strong());
            ui.add(egui::Slider::new(&mut state.smith_predictor.ki, 0.0..=5.0).step_by(0.001));
            ui.end_row();
            ui.label(RichText::new("Kd:").strong());
            ui.add(egui::Slider::new(&mut state.smith_predictor.kd, 0.0..=2.0).step_by(0.001));
            ui.end_row();
            ui.label(RichText::new("Setpoint:").strong());
            ui.add(
                egui::Slider::new(&mut state.smith_predictor.setpoint, -500.0..=500.0).step_by(0.1),
            );
            ui.end_row();
            ui.label("Output Limit:");
            ui.add(
                egui::Slider::new(&mut state.smith_predictor.output_limit, 1.0..=1000.0)
                    .step_by(1.0),
            );
            ui.end_row();
        });

    ui.add_space(8.0);
    ui.label(RichText::new(Tr::process_model(lang)).size(16.0).strong());
    ui.add_space(8.0);

    egui::Grid::new("smith_model_grid")
        .num_columns(2)
        .spacing([20.0, 10.0])
        .show(ui, |ui| {
            ui.label(format!("{}:", Tr::model_gain(lang)));
            ui.add(
                egui::Slider::new(&mut state.smith_predictor.model_gain, 0.1..=10.0).step_by(0.01),
            );
            ui.end_row();
            ui.label(format!("{}:", Tr::time_constant(lang)));
            ui.add(
                egui::Slider::new(&mut state.smith_predictor.model_time_const, 0.01..=10.0)
                    .step_by(0.01),
            );
            ui.end_row();
            ui.label(format!("{}:", Tr::dead_time(lang)));
            ui.add(
                egui::Slider::new(&mut state.smith_predictor.model_dead_time, 0.0..=5.0)
                    .step_by(0.01),
            );
            ui.end_row();
        });

    ui.add_space(8.0);
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(14.0, 8.0);
        ui.label(format!("{}:", Tr::model_prediction(lang)));
        ui.label(
            RichText::new(format!("{:.4}", state.smith_predictor.model_prediction()))
                .strong()
                .monospace(),
        );
        ui.label(format!("{}:", Tr::delay_buffer_size(lang)));
        ui.label(
            RichText::new(format!("{}", state.smith_predictor.delay_buffer_len()))
                .strong()
                .monospace(),
        );
    });
}

// ═══════════════════════════════════════════════════════════════
// ADRC 参数面板
// ═══════════════════════════════════════════════════════════════

fn show_adrc(ui: &mut Ui, state: &mut AppState) {
    let lang = state.lang();

    ui.label(RichText::new(Tr::adrc_td_params(lang)).size(16.0).strong());
    ui.add_space(8.0);

    egui::Grid::new("adrc_td_grid")
        .num_columns(2)
        .spacing([20.0, 10.0])
        .show(ui, |ui| {
            ui.label(RichText::new("TD r:").strong());
            ui.add(
                egui::Slider::new(&mut state.adrc.td_r, 1.0..=10000.0)
                    .step_by(1.0)
                    .logarithmic(true),
            );
            ui.end_row();
            ui.label(RichText::new("TD h0:").strong());
            ui.add(egui::Slider::new(&mut state.adrc.td_h0, 0.001..=0.5).step_by(0.001));
            ui.end_row();
        });

    ui.add_space(8.0);
    ui.label(RichText::new(Tr::adrc_eso_params(lang)).size(16.0).strong());
    ui.add_space(8.0);

    egui::Grid::new("adrc_eso_grid")
        .num_columns(2)
        .spacing([20.0, 10.0])
        .show(ui, |ui| {
            ui.label(RichText::new("ESO β1:").strong());
            ui.add(
                egui::Slider::new(&mut state.adrc.eso_beta1, 1.0..=2000.0)
                    .step_by(1.0)
                    .logarithmic(true),
            );
            ui.end_row();
            ui.label(RichText::new("ESO β2:").strong());
            ui.add(
                egui::Slider::new(&mut state.adrc.eso_beta2, 1.0..=50000.0)
                    .step_by(1.0)
                    .logarithmic(true),
            );
            ui.end_row();
            ui.label(RichText::new("ESO β3:").strong());
            ui.add(
                egui::Slider::new(&mut state.adrc.eso_beta3, 1.0..=200000.0)
                    .step_by(1.0)
                    .logarithmic(true),
            );
            ui.end_row();
            ui.label(RichText::new("ESO b0:").strong());
            ui.add(egui::Slider::new(&mut state.adrc.eso_b0, 0.1..=100.0).step_by(0.1));
            ui.end_row();
        });

    ui.add_space(8.0);
    ui.label(
        RichText::new(Tr::adrc_nlsef_params(lang))
            .size(16.0)
            .strong(),
    );
    ui.add_space(8.0);

    egui::Grid::new("adrc_nlsef_grid")
        .num_columns(2)
        .spacing([20.0, 10.0])
        .show(ui, |ui| {
            ui.label(RichText::new("β1:").strong());
            ui.add(egui::Slider::new(&mut state.adrc.nlsef_beta1, 0.0..=100.0).step_by(0.1));
            ui.end_row();
            ui.label(RichText::new("β2:").strong());
            ui.add(egui::Slider::new(&mut state.adrc.nlsef_beta2, 0.0..=50.0).step_by(0.1));
            ui.end_row();
            ui.label(RichText::new("α1:").strong());
            ui.add(egui::Slider::new(&mut state.adrc.nlsef_alpha1, 0.01..=1.0).step_by(0.01));
            ui.end_row();
            ui.label(RichText::new("α2:").strong());
            ui.add(egui::Slider::new(&mut state.adrc.nlsef_alpha2, 0.5..=3.0).step_by(0.01));
            ui.end_row();
            ui.label(RichText::new("δ:").strong());
            ui.add(egui::Slider::new(&mut state.adrc.nlsef_delta, 0.001..=1.0).step_by(0.001));
            ui.end_row();
        });

    ui.add_space(4.0);
    ui.label(RichText::new("Setpoint:").strong());
    ui.add(egui::Slider::new(&mut state.adrc.setpoint, -500.0..=500.0).step_by(0.1));
    ui.label(RichText::new("Output Limit:").strong());
    ui.add(egui::Slider::new(&mut state.adrc.output_limit, 1.0..=1000.0).step_by(1.0));
}

// ═══════════════════════════════════════════════════════════════
// LADRC 参数面板
// ═══════════════════════════════════════════════════════════════

fn show_ladrc(ui: &mut Ui, state: &mut AppState) {
    let lang = state.lang();

    ui.label(
        RichText::new(Tr::ladrc_bandwidth_params(lang))
            .size(16.0)
            .strong(),
    );
    ui.add_space(8.0);

    egui::Grid::new("ladrc_params_grid")
        .num_columns(2)
        .spacing([20.0, 10.0])
        .show(ui, |ui| {
            ui.label(RichText::new(format!("{}:", Tr::ladrc_order(lang))).strong());
            let mut order_idx: usize = match state.ladrc.order {
                crate::models::ladrc::LadrcOrder::First => 0,
                crate::models::ladrc::LadrcOrder::Second => 1,
            };
            ui.horizontal(|ui| {
                ui.selectable_value(&mut order_idx, 0, Tr::ladrc_first_order(lang));
                ui.selectable_value(&mut order_idx, 1, Tr::ladrc_second_order(lang));
            });
            state.ladrc.order = if order_idx == 0 {
                crate::models::ladrc::LadrcOrder::First
            } else {
                crate::models::ladrc::LadrcOrder::Second
            };
            ui.end_row();

            ui.label(RichText::new("ωc (Controller):").strong());
            ui.add(
                egui::Slider::new(&mut state.ladrc.omega_c, 0.1..=200.0)
                    .step_by(0.1)
                    .logarithmic(true),
            );
            ui.end_row();
            ui.label(RichText::new("ωo (Observer):").strong());
            ui.add(
                egui::Slider::new(&mut state.ladrc.omega_o, 0.1..=1000.0)
                    .step_by(0.1)
                    .logarithmic(true),
            );
            ui.end_row();
            ui.label(RichText::new("b0:").strong());
            ui.add(egui::Slider::new(&mut state.ladrc.b0, 0.1..=100.0).step_by(0.1));
            ui.end_row();
        });

    ui.add_space(4.0);
    ui.label(RichText::new("Setpoint:").strong());
    ui.add(egui::Slider::new(&mut state.ladrc.setpoint, -500.0..=500.0).step_by(0.1));
    ui.label(RichText::new("Output Limit:").strong());
    ui.add(egui::Slider::new(&mut state.ladrc.output_limit, 1.0..=1000.0).step_by(1.0));
}

// ═══════════════════════════════════════════════════════════════
// LQR 参数面板
// ═══════════════════════════════════════════════════════════════

fn show_lqr(ui: &mut Ui, state: &mut AppState) {
    let lang = state.lang();

    ui.label(RichText::new(Tr::lqr_weights(lang)).size(16.0).strong());
    ui.add_space(8.0);

    egui::Grid::new("lqr_params_grid")
        .num_columns(2)
        .spacing([20.0, 10.0])
        .show(ui, |ui| {
            ui.label(RichText::new(format!("{} (Q1):", Tr::lqr_q_position(lang))).strong());
            ui.add(
                egui::Slider::new(&mut state.lqr.q1, 0.01..=1000.0)
                    .step_by(0.01)
                    .logarithmic(true),
            );
            ui.end_row();
            ui.label(RichText::new(format!("{} (Q2):", Tr::lqr_q_velocity(lang))).strong());
            ui.add(
                egui::Slider::new(&mut state.lqr.q2, 0.01..=100.0)
                    .step_by(0.01)
                    .logarithmic(true),
            );
            ui.end_row();
            ui.label(RichText::new(format!("{} (R):", Tr::lqr_r_weight(lang))).strong());
            ui.add(
                egui::Slider::new(&mut state.lqr.r_weight, 0.001..=100.0)
                    .step_by(0.001)
                    .logarithmic(true),
            );
            ui.end_row();
            ui.label(RichText::new(format!("{}:", Tr::lqr_mass(lang))).strong());
            ui.add(egui::Slider::new(&mut state.lqr.mass, 0.1..=100.0).step_by(0.1));
            ui.end_row();
        });

    ui.add_space(8.0);
    ui.label(RichText::new(Tr::lqr_integral(lang)).size(14.0).strong());
    ui.add_space(4.0);

    egui::Grid::new("lqr_integral_grid")
        .num_columns(2)
        .spacing([20.0, 10.0])
        .show(ui, |ui| {
            ui.label(RichText::new("Ki:").strong());
            ui.add(egui::Slider::new(&mut state.lqr.ki, 0.0..=5.0).step_by(0.001));
            ui.end_row();
            ui.label(RichText::new("Integral Limit:").strong());
            ui.add(egui::Slider::new(&mut state.lqr.integral_limit, 0.0..=500.0).step_by(1.0));
            ui.end_row();
        });

    // 显示计算出的增益
    state.lqr.compute_gains();
    let k1 = state.lqr.k1;
    let k2 = state.lqr.k2;
    ui.add_space(8.0);
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(14.0, 8.0);
        ui.label(format!("{} K1:", Tr::lqr_computed_gains(lang)));
        ui.label(RichText::new(format!("{:.4}", k1)).strong().monospace());
        ui.label("K2:");
        ui.label(RichText::new(format!("{:.4}", k2)).strong().monospace());
    });

    ui.add_space(4.0);
    ui.label(RichText::new("Setpoint:").strong());
    ui.add(egui::Slider::new(&mut state.lqr.setpoint, -500.0..=500.0).step_by(0.1));
    ui.label(RichText::new("Output Limit:").strong());
    ui.add(egui::Slider::new(&mut state.lqr.output_limit, 1.0..=1000.0).step_by(1.0));
}

// ═══════════════════════════════════════════════════════════════
// MPC 参数面板
// ═══════════════════════════════════════════════════════════════

fn show_mpc(ui: &mut Ui, state: &mut AppState) {
    let lang = state.lang();

    ui.label(RichText::new(Tr::mpc_horizons(lang)).size(16.0).strong());
    ui.add_space(8.0);

    egui::Grid::new("mpc_horizon_grid")
        .num_columns(2)
        .spacing([20.0, 10.0])
        .show(ui, |ui| {
            ui.label(RichText::new(format!("{}:", Tr::mpc_prediction_horizon(lang))).strong());
            ui.add(egui::Slider::new(&mut state.mpc.prediction_horizon, 2..=50));
            ui.end_row();
            ui.label(RichText::new(format!("{}:", Tr::mpc_control_horizon(lang))).strong());
            ui.add(egui::Slider::new(&mut state.mpc.control_horizon, 1..=20));
            ui.end_row();
        });

    ui.add_space(8.0);
    ui.label(
        RichText::new(Tr::mpc_model_params(lang))
            .size(16.0)
            .strong(),
    );
    ui.add_space(8.0);

    egui::Grid::new("mpc_model_grid")
        .num_columns(2)
        .spacing([20.0, 10.0])
        .show(ui, |ui| {
            ui.label(format!("{}:", Tr::model_gain(lang)));
            ui.add(egui::Slider::new(&mut state.mpc.model_gain, 0.1..=10.0).step_by(0.01));
            ui.end_row();
            ui.label(format!("{}:", Tr::time_constant(lang)));
            ui.add(egui::Slider::new(&mut state.mpc.model_time_const, 0.01..=10.0).step_by(0.01));
            ui.end_row();
            ui.label(format!("{}:", Tr::mpc_sample_time(lang)));
            ui.add(egui::Slider::new(&mut state.mpc.sample_time, 0.001..=1.0).step_by(0.001));
            ui.end_row();
        });

    ui.add_space(8.0);
    ui.label(
        RichText::new(Tr::mpc_weights_and_constraints(lang))
            .size(16.0)
            .strong(),
    );
    ui.add_space(8.0);

    egui::Grid::new("mpc_weights_grid")
        .num_columns(2)
        .spacing([20.0, 10.0])
        .show(ui, |ui| {
            ui.label(RichText::new("Q (output):").strong());
            ui.add(
                egui::Slider::new(&mut state.mpc.q_weight, 0.01..=100.0)
                    .step_by(0.01)
                    .logarithmic(true),
            );
            ui.end_row();
            ui.label(RichText::new("R (input):").strong());
            ui.add(
                egui::Slider::new(&mut state.mpc.r_weight, 0.001..=10.0)
                    .step_by(0.001)
                    .logarithmic(true),
            );
            ui.end_row();
            ui.label(RichText::new("S (rate):").strong());
            ui.add(egui::Slider::new(&mut state.mpc.s_weight, 0.0..=10.0).step_by(0.001));
            ui.end_row();
            ui.label(format!("{}:", Tr::mpc_du_limit(lang)));
            ui.add(egui::Slider::new(&mut state.mpc.du_limit, 0.1..=100.0).step_by(0.1));
            ui.end_row();
        });

    ui.add_space(4.0);
    ui.label(RichText::new("Setpoint:").strong());
    ui.add(egui::Slider::new(&mut state.mpc.setpoint, -500.0..=500.0).step_by(0.1));
    ui.label(RichText::new("Output Limit:").strong());
    ui.add(egui::Slider::new(&mut state.mpc.output_limit, 1.0..=1000.0).step_by(1.0));
}
