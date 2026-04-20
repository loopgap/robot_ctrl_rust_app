use crate::app::AppState;
use egui::{Color32, RichText, Ui};
use egui_plot::{Line, Plot, PlotPoints};

/// 数据可视化视图 - 实时折线图
pub fn show(ui: &mut Ui, state: &mut AppState) {
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());

            // 图表开关控制
            show_chart_controls(ui, state);
            ui.add_space(12.0);

            // 各个图表
            if state.ui_state.show_position {
                show_chart(
                    ui,
                    "Position Over Time",
                    "position_chart",
                    &state.chart_data_position(),
                    Color32::from_rgb(33, 150, 243),
                );
                ui.add_space(12.0);
            }

            if state.ui_state.show_velocity {
                show_chart(
                    ui,
                    "Velocity Over Time",
                    "velocity_chart",
                    &state.chart_data_velocity(),
                    Color32::from_rgb(76, 175, 80),
                );
                ui.add_space(12.0);
            }

            if state.ui_state.show_current {
                show_chart(
                    ui,
                    "Current Over Time",
                    "current_chart",
                    &state.chart_data_current(),
                    Color32::from_rgb(255, 152, 0),
                );
                ui.add_space(12.0);
            }

            if state.ui_state.show_temperature {
                show_chart(
                    ui,
                    "Temperature Over Time",
                    "temperature_chart",
                    &state.chart_data_temperature(),
                    Color32::from_rgb(244, 67, 54),
                );
                ui.add_space(12.0);
            }

            if state.ui_state.show_error {
                show_chart(
                    ui,
                    "Error Over Time",
                    "error_chart",
                    &state.chart_data_error(),
                    Color32::from_rgb(156, 39, 176),
                );
            }
        });
}

/// 图表控制开关
fn show_chart_controls(ui: &mut Ui, state: &mut AppState) {
    egui::Frame::group(ui.style())
        .inner_margin(12.0)
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.heading("Chart Controls");
            ui.add_space(8.0);

            ui.horizontal_wrapped(|ui| {
                ui.toggle_value(
                    &mut state.ui_state.show_position,
                    RichText::new("📍 Position").color(if state.ui_state.show_position {
                        Color32::from_rgb(33, 150, 243)
                    } else {
                        Color32::GRAY
                    }),
                );

                ui.toggle_value(
                    &mut state.ui_state.show_velocity,
                    RichText::new("🏃 Velocity").color(if state.ui_state.show_velocity {
                        Color32::from_rgb(76, 175, 80)
                    } else {
                        Color32::GRAY
                    }),
                );

                ui.toggle_value(
                    &mut state.ui_state.show_current,
                    RichText::new("⚡ Current").color(if state.ui_state.show_current {
                        Color32::from_rgb(255, 152, 0)
                    } else {
                        Color32::GRAY
                    }),
                );

                ui.toggle_value(
                    &mut state.ui_state.show_temperature,
                    RichText::new("🌡 Temperature").color(if state.ui_state.show_temperature {
                        Color32::from_rgb(244, 67, 54)
                    } else {
                        Color32::GRAY
                    }),
                );

                ui.toggle_value(
                    &mut state.ui_state.show_error,
                    RichText::new("❌ Error").color(if state.ui_state.show_error {
                        Color32::from_rgb(156, 39, 176)
                    } else {
                        Color32::GRAY
                    }),
                );
            });

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(format!("Data points: {}", state.state_history.len()));
                if ui.button("🗑 Clear History").clicked() {
                    state.state_history.clear();
                }
            });
        });
}

/// 绘制折线图
fn show_chart(ui: &mut Ui, title: &str, id: &str, data: &[(f64, f64)], color: Color32) {
    egui::Frame::group(ui.style())
        .inner_margin(12.0)
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.label(RichText::new(title).strong().size(14.0));
            ui.add_space(4.0);

            let points: PlotPoints = data.iter().map(|&(x, y)| [x, y]).collect();
            let line = Line::new(points).color(color).width(2.0).name(title);

            Plot::new(id)
                .height(200.0)
                .allow_zoom(true)
                .allow_drag(true)
                .allow_scroll(true)
                .show_axes(true)
                .show_grid(true)
                .auto_bounds([true, true].into())
                .show(ui, |plot_ui| {
                    plot_ui.line(line);
                });
        });
}
