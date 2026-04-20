use crate::i18n::Language;
use eframe::egui;

#[derive(Clone, Copy)]
pub enum LoopState {
    Pending,
    Done,
    Warning,
}

pub struct LoopStep {
    pub name: &'static str,
    pub state: LoopState,
    pub detail: String,
}

pub fn render_loop_panel(ui: &mut egui::Ui, steps: &[LoopStep], lang: Language) {
    match lang {
        Language::Zh => {
            ui.heading("闭环流程");
            ui.label("输入 → 校验 → 执行 → 验证 → 导出");
        }
        Language::En => {
            ui.heading("Closed Loop");
            ui.label("Input → Validate → Execute → Verify → Export");
        }
    }
    ui.separator();

    for step in steps {
        let (icon, color) = match step.state {
            LoopState::Pending => ("○", egui::Color32::GRAY),
            LoopState::Done => ("●", egui::Color32::LIGHT_GREEN),
            LoopState::Warning => ("▲", egui::Color32::YELLOW),
        };

        egui::Frame::group(ui.style())
            .fill(egui::Color32::from_rgb(26, 26, 26))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.colored_label(color, icon);
                    ui.strong(step.name);
                });
                ui.label(&step.detail);
            });
        ui.add_space(4.0);
    }
}
