use crate::app::{simulation_lab::SimulationLabState, AppState};
use crate::i18n::Tr;
use crate::views::ui_kit::{page_header, section_title, settings_card};
use egui::{self, RichText, Ui};
use egui_plot::{Line, Plot, PlotPoints};

pub fn show(ui: &mut Ui, state: &mut AppState) {
    let _theme = state.theme.clone();
    let lang = state.lang();
    state.simulation.poll();

    page_header(ui, Tr::tab_simulation_lab(lang), "simulation");
    egui::ScrollArea::vertical()
        .id_salt("simulation_lab_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            settings_card(ui, |ui| {
                section_title(ui, Tr::simulation_scenario(lang));
                show_scenario_controls(ui, &mut state.simulation, lang);
            });

            ui.add_space(10.0);
            settings_card(ui, |ui| {
                section_title(ui, Tr::simulation_progress(lang));
                show_run_controls(ui, &mut state.simulation, lang);
            });

            ui.add_space(10.0);
            settings_card(ui, |ui| {
                section_title(ui, Tr::simulation_results(lang));
                show_metrics(ui, &state.simulation, lang);
            });

            ui.add_space(10.0);
            settings_card(ui, |ui| {
                section_title(ui, Tr::simulation_scan(lang));
                show_scan_controls(ui, &mut state.simulation, lang);
            });

            ui.add_space(10.0);
            settings_card(ui, |ui| {
                section_title(ui, Tr::simulation_export_preview(lang));
                show_export_preview(ui, &state.simulation, lang);
            });
        });
}

fn show_scenario_controls(ui: &mut Ui, sim: &mut SimulationLabState, lang: crate::i18n::Language) {
    egui::Grid::new("simulation_config_grid")
        .num_columns(2)
        .spacing([16.0, 8.0])
        .show(ui, |ui| {
            ui.label(Tr::simulation_duration(lang));
            ui.add_enabled(
                !sim.running,
                egui::TextEdit::singleline(&mut sim.duration_text).desired_width(120.0),
            );
            ui.end_row();

            ui.label(Tr::simulation_step_us(lang));
            ui.add_enabled(
                !sim.running,
                egui::TextEdit::singleline(&mut sim.dt_us_text).desired_width(120.0),
            );
            ui.end_row();

            ui.label(Tr::simulation_speed_ref(lang));
            ui.add_enabled(
                !sim.running,
                egui::TextEdit::singleline(&mut sim.speed_ref_text).desired_width(120.0),
            );
            ui.end_row();

            ui.label(Tr::simulation_load_torque(lang));
            ui.add_enabled(
                !sim.running,
                egui::TextEdit::singleline(&mut sim.load_torque_text).desired_width(120.0),
            );
            ui.end_row();
        });
}

fn show_run_controls(ui: &mut Ui, sim: &mut SimulationLabState, lang: crate::i18n::Language) {
    ui.horizontal_wrapped(|ui| {
        if ui
            .add_enabled(!sim.running, egui::Button::new(Tr::simulation_run(lang)))
            .clicked()
        {
            if let Err(err) = sim.start_run() {
                sim.status = format!("Failed: {err}");
            }
        }
        if ui
            .add_enabled(
                sim.can_cancel(),
                egui::Button::new(Tr::simulation_cancel(lang)),
            )
            .clicked()
        {
            sim.cancel();
        }
        ui.separator();
        ui.label(format!("{}: {}", Tr::simulation_status(lang), sim.status));
    });
    ui.add_space(8.0);
    ui.add(
        egui::ProgressBar::new(sim.progress.clamp(0.0, 1.0))
            .desired_width(f32::INFINITY)
            .show_percentage(),
    );
}

fn show_metrics(ui: &mut Ui, sim: &SimulationLabState, lang: crate::i18n::Language) {
    if let Some(result) = &sim.result {
        let metrics = &result.metrics;
        egui::Grid::new("simulation_metrics_grid")
            .num_columns(4)
            .spacing([18.0, 8.0])
            .striped(true)
            .show(ui, |ui| {
                metric(
                    ui,
                    "Final speed",
                    format!("{:.3} rad/s", metrics.final_speed),
                );
                metric(
                    ui,
                    "Speed error",
                    format!("{:.3}%", metrics.speed_error_pct),
                );
                ui.end_row();
                metric(ui, "Peak torque", format!("{:.3} N m", metrics.peak_torque));
                metric(ui, "Peak current", format!("{:.3} A", metrics.peak_current));
                ui.end_row();
                metric(
                    ui,
                    "Max temperature",
                    format!("{:.2} C", metrics.max_temperature_c),
                );
                metric(ui, "Steps executed", metrics.steps_executed.to_string());
                ui.end_row();
                metric(ui, "Settled", yes_no(metrics.settled));
                metric(ui, "Cancelled", yes_no(metrics.cancelled));
                ui.end_row();
            });

        ui.add_space(12.0);
        let speed_points: PlotPoints = result
            .samples
            .iter()
            .map(|sample| [sample.time_s, sample.omega_m])
            .collect();
        let ref_points: PlotPoints = result
            .samples
            .iter()
            .map(|sample| [sample.time_s, sample.speed_ref])
            .collect();
        Plot::new("simulation_speed_plot")
            .height(220.0)
            .legend(egui_plot::Legend::default())
            .show(ui, |plot_ui| {
                plot_ui.line(Line::new(speed_points).name("omega_m"));
                plot_ui.line(Line::new(ref_points).name("speed_ref"));
            });
    } else {
        ui.label(RichText::new(Tr::simulation_no_result(lang)).weak());
    }
}

fn show_scan_controls(ui: &mut Ui, sim: &mut SimulationLabState, lang: crate::i18n::Language) {
    ui.horizontal_wrapped(|ui| {
        let params = SimulationLabState::scan_params();
        let selected = params
            .get(sim.scan_param_idx)
            .map(|(_, label)| *label)
            .unwrap_or("Speed reference");
        egui::ComboBox::from_id_salt("simulation_scan_param")
            .selected_text(selected)
            .width(190.0)
            .show_ui(ui, |ui| {
                for (idx, (_, label)) in params.iter().enumerate() {
                    ui.selectable_value(&mut sim.scan_param_idx, idx, *label);
                }
            });
        ui.add_enabled(
            !sim.running,
            egui::TextEdit::singleline(&mut sim.scan_values_text)
                .desired_width(240.0)
                .hint_text("60,100,140"),
        );
        if ui
            .add_enabled(!sim.running, egui::Button::new(Tr::simulation_scan(lang)))
            .clicked()
        {
            if let Err(err) = sim.start_scan() {
                sim.status = format!("Scan failed: {err}");
            }
        }
    });

    if !sim.scan_results.is_empty() {
        ui.add_space(10.0);
        egui::Grid::new("simulation_scan_grid")
            .num_columns(5)
            .spacing([16.0, 6.0])
            .striped(true)
            .show(ui, |ui| {
                ui.strong("Value");
                ui.strong("Final speed");
                ui.strong("Error");
                ui.strong("Peak torque");
                ui.strong("Settled");
                ui.end_row();
                for point in &sim.scan_results {
                    ui.label(format!("{:.4}", point.param_value));
                    ui.label(format!("{:.3}", point.final_speed));
                    ui.label(format!("{:.3}%", point.speed_error_pct));
                    ui.label(format!("{:.3}", point.peak_torque));
                    ui.label(yes_no(point.settled));
                    ui.end_row();
                }
            });
    }
}

fn show_export_preview(ui: &mut Ui, sim: &SimulationLabState, lang: crate::i18n::Language) {
    if sim.result.is_none() {
        ui.label(RichText::new(Tr::simulation_export_empty(lang)).weak());
        return;
    }

    ui.label("CSV");
    let mut csv_preview = preview_text(&sim.export_csv_text(), 1_800);
    ui.add(
        egui::TextEdit::multiline(&mut csv_preview)
            .desired_rows(7)
            .desired_width(f32::INFINITY)
            .interactive(false),
    );

    ui.add_space(8.0);
    ui.label("JSON");
    let mut json_preview = preview_text(&sim.export_json_text(), 1_800);
    ui.add(
        egui::TextEdit::multiline(&mut json_preview)
            .desired_rows(8)
            .desired_width(f32::INFINITY)
            .interactive(false),
    );
}

fn metric(ui: &mut Ui, label: &str, value: String) {
    ui.label(RichText::new(label).weak());
    ui.label(value);
}

fn yes_no(value: bool) -> String {
    if value {
        "yes".into()
    } else {
        "no".into()
    }
}

fn preview_text(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_owned();
    }
    let mut preview: String = text.chars().take(max_chars).collect();
    preview.push_str("\n...");
    preview
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_text_truncates_by_chars() {
        let preview = preview_text("abcdef", 3);
        assert_eq!(preview, "abc\n...");
    }

    #[test]
    fn yes_no_is_plain_text() {
        assert_eq!(yes_no(true), "yes");
        assert_eq!(yes_no(false), "no");
    }
}
