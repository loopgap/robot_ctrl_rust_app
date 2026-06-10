use crate::app::AppState;
use crate::i18n::Tr;
use crate::views::ui_kit::{page_header, settings_card};
use egui::{self, Color32, RichText, Ui};
use egui_plot::{Line, Plot, PlotPoints};

const NN_LABEL_WIDTH: f32 = 120.0;

pub fn show(ui: &mut Ui, state: &mut AppState) {
    let theme = state.theme.clone();
    let current_time = ui.ctx().input(|i| i.time);
    let lang = state.lang();
    page_header(ui, Tr::tab_nn_tuning(lang), "nn");

    settings_card(ui, |ui| {
        ui.label(RichText::new(Tr::network_arch(lang)).size(15.0).strong());
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            let arch: Vec<String> = state
                .control
                .nn
                .layers
                .iter()
                .enumerate()
                .map(|(i, layer)| {
                    if i == 0 {
                        format!(
                            "{}\u{2192}{} ({:?})",
                            layer.weights[0].len(),
                            layer.weights.len(),
                            layer.activation
                        )
                    } else {
                        format!("\u{2192}{} ({:?})", layer.weights.len(), layer.activation)
                    }
                })
                .collect();
            ui.label(format!("Layers: Input(6) {}", arch.join(" ")));
        });

        ui.horizontal(|ui| {
            ui.label(format!(
                "Training Epochs: {}  |  Loss History: {} points",
                state.control.nn.training_epochs,
                state.control.nn.loss_history.len()
            ));
        });
    });

    ui.add_space(10.0);

    // ─── 训练控制 ────────────────────────────────────────
    settings_card(ui, |ui| {
        ui.label(
            RichText::new(Tr::training_controls(lang))
                .size(15.0)
                .strong(),
        );
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            ui.label(format!("{}:", Tr::learning_rate(lang)));
            ui.add(
                egui::TextEdit::singleline(&mut state.ui.nn_learning_rate_text)
                    .desired_width(110.0),
            );
            if let Ok(lr) = state.ui.nn_learning_rate_text.parse::<f64>() {
                state.control.nn.learning_rate = lr;
            }
        });

        ui.add_space(8.0);

        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(10.0, 8.0);

            if ui
                .button(RichText::new(Tr::train_step(lang)).size(14.0))
                .clicked()
            {
                state.control.nn_train_step();
            }

            if ui
                .button(RichText::new(Tr::train_x10_btn(lang)).size(14.0))
                .clicked()
            {
                for _ in 0..10 {
                    state.control.nn_train_step();
                }
            }

            if ui
                .button(RichText::new(Tr::train_x100_btn(lang)).size(14.0))
                .clicked()
            {
                for _ in 0..100 {
                    state.control.nn_train_step();
                }
            }

            ui.checkbox(&mut state.ui.nn_auto_train, Tr::auto_train(lang));
        });
    });

    if state.ui.nn_auto_train && state.control.is_running {
        state.ui.nn_train_frame_counter += 1;
        if state.ui.nn_train_frame_counter.is_multiple_of(10) {
            state.control.nn_train_step();
        }
    }

    ui.add_space(10.0);

    // ─── Loss 曲线 ───────────────────────────────────────
    settings_card(ui, |ui| {
        ui.label(RichText::new(Tr::training_loss(lang)).size(15.0).strong());
        ui.add_space(8.0);

        if !state.control.nn.loss_history.is_empty() {
            let loss_points: PlotPoints = state
                .control
                .nn
                .loss_history
                .iter()
                .enumerate()
                .map(|(i, &loss)| [i as f64, loss])
                .collect();

            let loss_line = Line::new(loss_points)
                .name(Tr::loss_label(lang))
                .color(state.anim.animate_color(
                    "nn_tuning_1".into(),
                    theme.accent_orange,
                    theme.accent_orange,
                    0.3,
                    crate::app::animation::Easing::EaseOut,
                    current_time,
                ))
                .width(1.5);

            Plot::new("loss_plot")
                .height(170.0)
                .allow_drag(false)
                .allow_zoom(false)
                .show_axes(true)
                .show(ui, |plot_ui| {
                    plot_ui.line(loss_line);
                });

            ui.add_space(4.0);
            ui.label(
                RichText::new(format!(
                    "Current Loss: {:.6}",
                    state.control.nn.loss_history.last().unwrap_or(&0.0)
                ))
                .size(12.0)
                .color(state.anim.animate_color(
                    "nn_tuning_2".into(),
                    theme.accent_orange,
                    theme.accent_orange,
                    0.3,
                    crate::app::animation::Easing::EaseOut,
                    current_time,
                )),
            );
        } else {
            ui.add_space(8.0);
            crate::views::ui_kit::empty_state(
                ui,
                "nn",
                "No Training Data",
                Tr::no_training_data(lang),
                &theme,
            );
        }
    });

    ui.add_space(10.0);

    // ─── 建议参数 ────────────────────────────────────────
    settings_card(ui, |ui| {
        ui.label(
            RichText::new(Tr::suggested_params(lang))
                .size(15.0)
                .strong(),
        );
        ui.add_space(8.0);

        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(12.0, 8.0);

            if ui
                .button(RichText::new(Tr::predict(lang)).size(14.0))
                .clicked()
            {
                state.control.nn_suggest_params();
            }

            if ui
                .button(RichText::new(Tr::apply_suggested(lang)).size(14.0).color(
                    state.anim.animate_color(
                        "nn_tuning_1".into(),
                        theme.status_ok,
                        theme.status_ok,
                        0.3,
                        crate::app::animation::Easing::EaseOut,
                        current_time,
                    ),
                ))
                .clicked()
            {
                state.apply_nn_params();
            }
        });

        ui.add_space(10.0);

        egui::Grid::new("nn_suggest_grid")
            .num_columns(4)
            .spacing([28.0, 10.0])
            .show(ui, |ui| {
                ui.label(RichText::new(Tr::parameter(lang)).strong());
                ui.label(RichText::new(Tr::current(lang)).strong());
                ui.label(RichText::new(Tr::suggested(lang)).strong().color(
                    state.anim.animate_color(
                        "nn_tuning_1".into(),
                        theme.status_info,
                        theme.status_info,
                        0.3,
                        crate::app::animation::Easing::EaseOut,
                        current_time,
                    ),
                ));
                ui.label(RichText::new(Tr::delta(lang)).strong());
                ui.end_row();

                param_row(
                    ui,
                    "Kp",
                    state.control.pid().kp,
                    state.control.nn_suggested_kp,
                );
                param_row(
                    ui,
                    "Ki",
                    state.control.pid().ki,
                    state.control.nn_suggested_ki,
                );
                param_row(
                    ui,
                    "Kd",
                    state.control.pid().kd,
                    state.control.nn_suggested_kd,
                );
            });
    });

    ui.add_space(10.0);

    settings_card(ui, |ui| {
        ui.label(RichText::new("LLM API Tuning").size(15.0).strong());
        ui.add_space(8.0);

        egui::Grid::new("llm_tuning_grid")
            .num_columns(2)
            .spacing([18.0, 12.0])
            .show(ui, |ui| {
                ui.add_sized([NN_LABEL_WIDTH, 20.0], egui::Label::new("API URL:"));
                ui.add(egui::TextEdit::singleline(&mut state.ui.llm_api_url).desired_width(420.0));
                ui.end_row();

                ui.add_sized([NN_LABEL_WIDTH, 20.0], egui::Label::new("Model:"));
                ui.add(
                    egui::TextEdit::singleline(&mut state.ui.llm_model_name).desired_width(260.0),
                );
                ui.end_row();

                ui.add_sized([NN_LABEL_WIDTH, 20.0], egui::Label::new("API Key:"));
                ui.add(
                    egui::TextEdit::singleline(&mut state.ui.llm_api_key)
                        .password(true)
                        .desired_width(320.0),
                );
                ui.end_row();
            });

        ui.horizontal_wrapped(|ui| {
            let llm_btn = ui.add_enabled(
                !state.ui.llm_loading,
                egui::Button::new(RichText::new("LLM Suggest").size(14.0)),
            );
            if llm_btn.clicked() {
                state.llm_suggest_params();
            }
            if ui
                .button(RichText::new("Apply LLM Suggestion").size(14.0))
                .clicked()
            {
                state.apply_nn_params();
            }
        });

        if !state.ui.llm_last_response.is_empty() {
            ui.add_space(6.0);
            if state.ui.llm_loading {
                crate::views::ui_kit::loading_spinner(
                    ui,
                    &mut state.anim,
                    current_time,
                    "llm_loading",
                    "LLM request in progress...",
                    &theme,
                );
            }
            ui.label(RichText::new("LLM Analysis:").strong());
            ui.label(
                RichText::new(&state.ui.llm_last_response).color(state.anim.animate_color(
                    "nn_tuning_2".into(),
                    theme.status_info,
                    theme.status_info,
                    0.3,
                    crate::app::animation::Easing::EaseOut,
                    current_time,
                )),
            );
        }
    });

    ui.add_space(12.0);

    // ─── 特征预览 ────────────────────────────────────────
    if state.control.state_history.len() >= 10 {
        ui.collapsing(Tr::input_features(lang), |ui| {
            ui.add_space(4.0);
            let errors: Vec<f64> = state
                .control
                .state_history
                .iter()
                .map(|s| s.error)
                .collect();
            let features = crate::models::NeuralNetwork::extract_features(&errors);
            let labels = [
                "Mean Error",
                "Std Dev",
                "Oscillation",
                "Overshoot",
                "Steady State",
                "Rise Time",
            ];
            egui::Grid::new("features_grid")
                .num_columns(2)
                .spacing([20.0, 6.0])
                .show(ui, |ui| {
                    for (label, &val) in labels.iter().zip(features.iter()) {
                        ui.label(*label);
                        ui.label(RichText::new(format!("{:.4}", val)).monospace());
                        ui.end_row();
                    }
                });
        });
    }
}

fn param_row(ui: &mut Ui, name: &str, current: f64, suggested: f64) {
    let delta = suggested - current;
    let delta_color = if delta.abs() < 0.001 {
        Color32::GRAY
    } else if delta > 0.0 {
        Color32::from_rgb(100, 255, 100)
    } else {
        Color32::from_rgb(255, 100, 100)
    };

    ui.label(name);
    ui.label(RichText::new(format!("{:.4}", current)).monospace());
    ui.label(
        RichText::new(format!("{:.4}", suggested))
            .monospace()
            .color(Color32::from_rgb(100, 200, 255)),
    );
    ui.label(
        RichText::new(format!("{:+.4}", delta))
            .monospace()
            .color(delta_color),
    );
    ui.end_row();
}
