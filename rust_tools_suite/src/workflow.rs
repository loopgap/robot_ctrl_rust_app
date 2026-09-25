use crate::i18n::Language;
use crate::icons::{draw_tool_icon, ToolIconKind};
use crate::theme::ACCENT_COLOR;
use eframe::egui;

/// 步骤状态
#[derive(Clone, Copy)]
pub enum LoopState {
    Pending,
    Done,
    Warning,
}

/// 闭环流程步骤
///
/// 表示工作流中的一个步骤，包含名称、状态和详情。
pub struct LoopStep {
    pub name: &'static str,
    pub state: LoopState,
    pub detail: String,
}

pub fn render_loop_panel(ui: &mut egui::Ui, steps: &[LoopStep], lang: Language) {
    ui.horizontal(|ui| {
        let (icon_rect, _) = ui.allocate_exact_size(egui::vec2(20.0, 20.0), egui::Sense::hover());
        draw_tool_icon(
            ui.painter(),
            icon_rect,
            ToolIconKind::Workflow,
            ACCENT_COLOR,
        );
        match lang {
            Language::Zh => {
                ui.heading("闭环流程");
            }
            Language::En => {
                ui.heading("Closed Loop");
            }
        }
    });

    match lang {
        Language::Zh => {
            ui.label("输入 → 校验 → 执行 → 验证 → 导出");
        }
        Language::En => {
            ui.label("Input → Validate → Execute → Verify → Export");
        }
    }
    ui.separator();

    let bg_card = if ui.visuals().dark_mode {
        egui::Color32::from_rgb(26, 26, 26)
    } else {
        ui.visuals().faint_bg_color
    };

    for step in steps {
        egui::Frame::group(ui.style())
            .fill(bg_card)
            .corner_radius(6.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let (badge_rect, _) =
                        ui.allocate_exact_size(egui::vec2(14.0, 14.0), egui::Sense::hover());
                    let painter = ui.painter();
                    let center = badge_rect.center();

                    match step.state {
                        LoopState::Pending => {
                            painter.circle_stroke(
                                center,
                                4.5,
                                egui::Stroke::new(1.5_f32, egui::Color32::GRAY),
                            );
                        }
                        LoopState::Done => {
                            let ok_color = egui::Color32::from_rgb(52, 211, 153);
                            painter.circle_filled(center, 5.0, ok_color);
                            // Inner crisp dot
                            painter.circle_filled(center, 2.0, egui::Color32::WHITE);
                        }
                        LoopState::Warning => {
                            let warn_color = egui::Color32::from_rgb(251, 191, 36);
                            let pts = vec![
                                egui::pos2(center.x, center.y - 5.0),
                                egui::pos2(center.x + 5.0, center.y + 4.5),
                                egui::pos2(center.x - 5.0, center.y + 4.5),
                            ];
                            painter.add(egui::Shape::convex_polygon(
                                pts,
                                warn_color,
                                egui::Stroke::NONE,
                            ));
                        }
                    }

                    ui.strong(step.name);
                });
                ui.label(&step.detail);
            });
        ui.add_space(4.0);
    }
}
