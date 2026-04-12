use egui::{self, Color32, RichText, Ui};

pub fn apply_page_style(ui: &mut Ui) {
    let spacing = ui.spacing_mut();
    spacing.item_spacing = egui::vec2(14.0, 12.0);
    spacing.button_padding = egui::vec2(12.0, 8.0);
    spacing.interact_size.y = 34.0;
    spacing.text_edit_width = 260.0;
    spacing.combo_width = 240.0;
    spacing.slider_width = 300.0;
}

pub fn page_header(ui: &mut Ui, title: &str, icon: &str) {
    apply_page_style(ui);
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(22.0, 22.0), egui::Sense::hover());
        draw_header_icon(ui.painter(), rect, icon, ui.visuals().text_color());
        ui.add_space(8.0);
        ui.heading(RichText::new(title).size(24.0));
    });
    ui.add_space(12.0);
}

pub fn section_title(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).size(17.0).strong());
    ui.add_space(8.0);
}

pub fn settings_card(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) {
    egui::Frame::group(ui.style())
        .fill(ui.visuals().faint_bg_color)
        .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
        .corner_radius(12.0)
        .inner_margin(egui::Margin::symmetric(18, 16))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            add_contents(ui);
        });
}

fn draw_header_icon(painter: &egui::Painter, rect: egui::Rect, icon: &str, color: Color32) {
    let c = rect.center();
    let s = rect.width().min(rect.height());
    let stroke = egui::Stroke::new(1.5, color);
    match icon {
        "dashboard" => {
            for i in 0..3 {
                let w = s * 0.12;
                let x = rect.left() + s * (0.18 + i as f32 * 0.22);
                let h = s * (0.28 + i as f32 * 0.18);
                let r =
                    egui::Rect::from_min_size(egui::pos2(x, rect.bottom() - h), egui::vec2(w, h));
                painter.rect_filled(r, 1.0, color);
            }
        }
        "connections" => {
            painter.circle_stroke(egui::pos2(c.x - s * 0.16, c.y), s * 0.10, stroke);
            painter.circle_stroke(egui::pos2(c.x + s * 0.16, c.y), s * 0.10, stroke);
            painter.line_segment(
                [
                    egui::pos2(c.x - s * 0.06, c.y),
                    egui::pos2(c.x + s * 0.06, c.y),
                ],
                stroke,
            );
        }
        "terminal" => {
            let r = egui::Rect::from_center_size(c, egui::vec2(s * 0.58, s * 0.42));
            painter.rect_stroke(r, 2.0, stroke, egui::StrokeKind::Middle);
            painter.line_segment(
                [
                    egui::pos2(r.left() + 2.0, c.y),
                    egui::pos2(r.right() - 2.0, c.y),
                ],
                egui::Stroke::new(1.0, color),
            );
        }
        "packet" => {
            let r = egui::Rect::from_center_size(c, egui::vec2(s * 0.60, s * 0.46));
            painter.rect_stroke(r, 2.0, stroke, egui::StrokeKind::Middle);
            painter.line_segment(
                [
                    egui::pos2(r.left(), r.top() + s * 0.13),
                    egui::pos2(r.right(), r.top() + s * 0.13),
                ],
                stroke,
            );
        }
        "topology" => {
            let r = egui::Rect::from_center_size(c, egui::vec2(s * 0.60, s * 0.46));
            painter.rect_stroke(r, 4.0, stroke, egui::StrokeKind::Middle);
            painter.circle_filled(egui::pos2(c.x - s * 0.12, c.y - s * 0.07), 1.4, color);
            painter.circle_filled(egui::pos2(c.x + s * 0.12, c.y - s * 0.07), 1.4, color);
            painter.line_segment(
                [
                    egui::pos2(c.x - s * 0.14, c.y + s * 0.10),
                    egui::pos2(c.x + s * 0.14, c.y + s * 0.10),
                ],
                egui::Stroke::new(1.1, color),
            );
        }
        "pid" => {
            let r = egui::Rect::from_center_size(c, egui::vec2(s * 0.60, s * 0.42));
            painter.rect_stroke(r, 2.0, stroke, egui::StrokeKind::Middle);
            for i in 0..3 {
                let x = r.left() + r.width() * (0.24 + i as f32 * 0.26);
                painter.line_segment(
                    [
                        egui::pos2(x, r.top() + 2.0),
                        egui::pos2(x, r.bottom() - 2.0),
                    ],
                    egui::Stroke::new(1.0, color),
                );
            }
        }
        "nn" => {
            let n1 = egui::pos2(c.x - s * 0.20, c.y - s * 0.05);
            let n2 = egui::pos2(c.x - s * 0.20, c.y + s * 0.18);
            let n3 = egui::pos2(c.x + s * 0.02, c.y - s * 0.20);
            let n4 = egui::pos2(c.x + s * 0.20, c.y + s * 0.02);
            painter.line_segment([n1, n3], egui::Stroke::new(1.0, color));
            painter.line_segment([n2, n3], egui::Stroke::new(1.0, color));
            painter.line_segment([n3, n4], egui::Stroke::new(1.0, color));
            for p in [n1, n2, n3, n4] {
                painter.circle_filled(p, 1.7, color);
            }
        }
        "viz" => {
            let p0 = egui::pos2(c.x - s * 0.28, c.y + s * 0.16);
            let p1 = egui::pos2(c.x - s * 0.08, c.y - s * 0.02);
            let p2 = egui::pos2(c.x + s * 0.06, c.y + s * 0.08);
            let p3 = egui::pos2(c.x + s * 0.24, c.y - s * 0.18);
            painter.line_segment([p0, p1], stroke);
            painter.line_segment([p1, p2], stroke);
            painter.line_segment([p2, p3], stroke);
        }
        "modbus" => {
            let r = egui::Rect::from_center_size(c, egui::vec2(s * 0.60, s * 0.42));
            painter.rect_stroke(r, 2.0, stroke, egui::StrokeKind::Middle);
            painter.line_segment(
                [
                    egui::pos2(r.left(), r.center().y),
                    egui::pos2(r.right(), r.center().y),
                ],
                egui::Stroke::new(1.0, color),
            );
            painter.line_segment(
                [
                    egui::pos2(r.left() + s * 0.22, r.top()),
                    egui::pos2(r.left() + s * 0.22, r.bottom()),
                ],
                egui::Stroke::new(1.0, color),
            );
        }
        "canopen" => {
            let p1 = egui::pos2(c.x - s * 0.20, c.y);
            let p2 = egui::pos2(c.x, c.y - s * 0.18);
            let p3 = egui::pos2(c.x + s * 0.20, c.y);
            let p4 = egui::pos2(c.x, c.y + s * 0.18);
            painter.line_segment([p1, p2], egui::Stroke::new(1.1, color));
            painter.line_segment([p2, p3], egui::Stroke::new(1.1, color));
            painter.line_segment([p3, p4], egui::Stroke::new(1.1, color));
            painter.line_segment([p4, p1], egui::Stroke::new(1.1, color));
            for p in [p1, p2, p3, p4] {
                painter.circle_filled(p, 1.6, color);
            }
        }
        _ => {
            painter.rect_stroke(rect.shrink(1.0), 3.0, stroke, egui::StrokeKind::Middle);
        }
    }
}
