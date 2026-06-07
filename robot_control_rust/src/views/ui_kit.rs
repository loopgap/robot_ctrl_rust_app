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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconKind {
    Dashboard,
    Connections,
    Terminal,
    Packet,
    Topology,
    Pid,
    Neural,
    Visualization,
    Simulation,
    Modbus,
    Canopen,
    Line,
    Scatter,
    Bar,
    Gauge,
    Histogram,
    Table,
    Differential,
    Mecanum,
    Omni3,
    Omni4,
    Ackermann,
    Tracked,
    Scara,
    SixDofArm,
    DeltaRobot,
    Custom,
    Generic,
}

impl IconKind {
    pub fn from_key(key: &str) -> Self {
        match key {
            "dashboard" => Self::Dashboard,
            "connections" => Self::Connections,
            "terminal" => Self::Terminal,
            "packet" => Self::Packet,
            "topology" => Self::Topology,
            "pid" => Self::Pid,
            "nn" => Self::Neural,
            "viz" => Self::Visualization,
            "simulation" => Self::Simulation,
            "modbus" => Self::Modbus,
            "canopen" => Self::Canopen,
            "line" => Self::Line,
            "scatter" => Self::Scatter,
            "bar" => Self::Bar,
            "gauge" => Self::Gauge,
            "histogram" => Self::Histogram,
            "table" => Self::Table,
            "differential" => Self::Differential,
            "mecanum" => Self::Mecanum,
            "omni3" => Self::Omni3,
            "omni4" => Self::Omni4,
            "ackermann" => Self::Ackermann,
            "tracked" => Self::Tracked,
            "scara" => Self::Scara,
            "six_dof_arm" => Self::SixDofArm,
            "delta_robot" => Self::DeltaRobot,
            "custom" => Self::Custom,
            _ => Self::Generic,
        }
    }

    #[cfg(test)]
    fn key(self) -> &'static str {
        match self {
            Self::Dashboard => "dashboard",
            Self::Connections => "connections",
            Self::Terminal => "terminal",
            Self::Packet => "packet",
            Self::Topology => "topology",
            Self::Pid => "pid",
            Self::Neural => "nn",
            Self::Visualization => "viz",
            Self::Simulation => "simulation",
            Self::Modbus => "modbus",
            Self::Canopen => "canopen",
            Self::Line => "line",
            Self::Scatter => "scatter",
            Self::Bar => "bar",
            Self::Gauge => "gauge",
            Self::Histogram => "histogram",
            Self::Table => "table",
            Self::Differential => "differential",
            Self::Mecanum => "mecanum",
            Self::Omni3 => "omni3",
            Self::Omni4 => "omni4",
            Self::Ackermann => "ackermann",
            Self::Tracked => "tracked",
            Self::Scara => "scara",
            Self::SixDofArm => "six_dof_arm",
            Self::DeltaRobot => "delta_robot",
            Self::Custom => "custom",
            Self::Generic => "generic",
        }
    }
}

pub fn page_header(ui: &mut Ui, title: &str, icon: &str) {
    page_header_icon(ui, title, IconKind::from_key(icon));
}

pub fn page_header_icon(ui: &mut Ui, title: &str, icon: IconKind) {
    apply_page_style(ui);
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(22.0, 22.0), egui::Sense::hover());
        draw_icon(ui.painter(), rect, icon, ui.visuals().text_color());
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

pub fn draw_icon(painter: &egui::Painter, rect: egui::Rect, icon: IconKind, color: Color32) {
    let c = rect.center();
    let s = rect.width().min(rect.height());
    let stroke = egui::Stroke::new(1.5, color);
    match icon {
        IconKind::Dashboard => draw_bars(painter, rect, color),
        IconKind::Connections => {
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
        IconKind::Terminal => {
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
        IconKind::Packet => draw_panel(painter, c, s, stroke, true),
        IconKind::Topology => {
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
        IconKind::Pid => {
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
        IconKind::Neural => draw_neural(painter, c, s, color),
        IconKind::Visualization | IconKind::Line => draw_line_chart(painter, c, s, stroke),
        IconKind::Simulation => draw_simulation(painter, c, s, color, stroke),
        IconKind::Modbus => draw_modbus(painter, c, s, color, stroke),
        IconKind::Canopen => draw_canopen(painter, c, s, color),
        IconKind::Scatter => draw_scatter(painter, c, s, color),
        IconKind::Bar => draw_bars(painter, rect, color),
        IconKind::Gauge => draw_gauge(painter, c, s, color, stroke),
        IconKind::Histogram => draw_histogram(painter, rect, color),
        IconKind::Table => draw_table(painter, c, s, stroke),
        IconKind::Differential => draw_vehicle(painter, c, s, color, stroke, false),
        IconKind::Mecanum => draw_mecanum(painter, c, s, color, stroke),
        IconKind::Omni3 => draw_omni(painter, c, s, color, 3),
        IconKind::Omni4 => draw_omni(painter, c, s, color, 4),
        IconKind::Ackermann => draw_vehicle(painter, c, s, color, stroke, true),
        IconKind::Tracked => draw_tracked(painter, c, s, color, stroke),
        IconKind::Scara => draw_arm(painter, c, s, color, stroke, 2),
        IconKind::SixDofArm => draw_arm(painter, c, s, color, stroke, 3),
        IconKind::DeltaRobot => draw_delta(painter, c, s, color, stroke),
        IconKind::Custom | IconKind::Generic => {
            painter.rect_stroke(rect.shrink(1.0), 3.0, stroke, egui::StrokeKind::Middle);
            painter.line_segment(
                [
                    egui::pos2(c.x - s * 0.18, c.y),
                    egui::pos2(c.x + s * 0.18, c.y),
                ],
                stroke,
            );
        }
    }
}

fn draw_panel(painter: &egui::Painter, c: egui::Pos2, s: f32, stroke: egui::Stroke, header: bool) {
    let r = egui::Rect::from_center_size(c, egui::vec2(s * 0.60, s * 0.46));
    painter.rect_stroke(r, 2.0, stroke, egui::StrokeKind::Middle);
    if header {
        painter.line_segment(
            [
                egui::pos2(r.left(), r.top() + s * 0.13),
                egui::pos2(r.right(), r.top() + s * 0.13),
            ],
            stroke,
        );
    }
}

fn draw_bars(painter: &egui::Painter, rect: egui::Rect, color: Color32) {
    let s = rect.width().min(rect.height());
    for i in 0..3 {
        let w = s * 0.12;
        let x = rect.left() + s * (0.18 + i as f32 * 0.22);
        let h = s * (0.28 + i as f32 * 0.18);
        let r = egui::Rect::from_min_size(egui::pos2(x, rect.bottom() - h), egui::vec2(w, h));
        painter.rect_filled(r, 1.0, color);
    }
}

fn draw_line_chart(painter: &egui::Painter, c: egui::Pos2, s: f32, stroke: egui::Stroke) {
    let p0 = egui::pos2(c.x - s * 0.28, c.y + s * 0.16);
    let p1 = egui::pos2(c.x - s * 0.08, c.y - s * 0.02);
    let p2 = egui::pos2(c.x + s * 0.06, c.y + s * 0.08);
    let p3 = egui::pos2(c.x + s * 0.24, c.y - s * 0.18);
    painter.line_segment([p0, p1], stroke);
    painter.line_segment([p1, p2], stroke);
    painter.line_segment([p2, p3], stroke);
}

fn draw_neural(painter: &egui::Painter, c: egui::Pos2, s: f32, color: Color32) {
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

fn draw_simulation(
    painter: &egui::Painter,
    c: egui::Pos2,
    s: f32,
    color: Color32,
    stroke: egui::Stroke,
) {
    painter.circle_stroke(c, s * 0.18, stroke);
    painter.circle_filled(c, 1.8, color);
    for i in 0..3 {
        let angle = i as f32 * std::f32::consts::TAU / 3.0;
        let end = egui::pos2(c.x + angle.cos() * s * 0.18, c.y + angle.sin() * s * 0.18);
        painter.line_segment([c, end], egui::Stroke::new(1.0, color));
    }
    draw_line_chart(
        painter,
        egui::pos2(c.x, c.y + s * 0.04),
        s * 0.72,
        egui::Stroke::new(1.0, color),
    );
}

fn draw_modbus(
    painter: &egui::Painter,
    c: egui::Pos2,
    s: f32,
    color: Color32,
    stroke: egui::Stroke,
) {
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

fn draw_canopen(painter: &egui::Painter, c: egui::Pos2, s: f32, color: Color32) {
    let p1 = egui::pos2(c.x - s * 0.20, c.y);
    let p2 = egui::pos2(c.x, c.y - s * 0.18);
    let p3 = egui::pos2(c.x + s * 0.20, c.y);
    let p4 = egui::pos2(c.x, c.y + s * 0.18);
    for pair in [[p1, p2], [p2, p3], [p3, p4], [p4, p1]] {
        painter.line_segment(pair, egui::Stroke::new(1.1, color));
    }
    for p in [p1, p2, p3, p4] {
        painter.circle_filled(p, 1.6, color);
    }
}

fn draw_scatter(painter: &egui::Painter, c: egui::Pos2, s: f32, color: Color32) {
    for (x, y) in [(-0.22, 0.14), (-0.06, -0.12), (0.10, 0.04), (0.24, -0.18)] {
        painter.circle_filled(egui::pos2(c.x + s * x, c.y + s * y), 1.8, color);
    }
}

fn draw_gauge(
    painter: &egui::Painter,
    c: egui::Pos2,
    s: f32,
    color: Color32,
    stroke: egui::Stroke,
) {
    painter.circle_stroke(egui::pos2(c.x, c.y + s * 0.08), s * 0.24, stroke);
    painter.line_segment(
        [
            egui::pos2(c.x, c.y + s * 0.08),
            egui::pos2(c.x + s * 0.16, c.y - s * 0.08),
        ],
        egui::Stroke::new(1.2, color),
    );
}

fn draw_histogram(painter: &egui::Painter, rect: egui::Rect, color: Color32) {
    let s = rect.width().min(rect.height());
    for i in 0..5 {
        let w = s * 0.08;
        let x = rect.left() + s * (0.14 + i as f32 * 0.13);
        let h = s * (0.18 + ((i + 2) % 3) as f32 * 0.10);
        painter.rect_filled(
            egui::Rect::from_min_size(egui::pos2(x, rect.bottom() - h), egui::vec2(w, h)),
            0.8,
            color,
        );
    }
}

fn draw_table(painter: &egui::Painter, c: egui::Pos2, s: f32, stroke: egui::Stroke) {
    let r = egui::Rect::from_center_size(c, egui::vec2(s * 0.58, s * 0.44));
    painter.rect_stroke(r, 1.0, stroke, egui::StrokeKind::Middle);
    for frac in [0.33, 0.66] {
        let x = r.left() + r.width() * frac;
        painter.line_segment([egui::pos2(x, r.top()), egui::pos2(x, r.bottom())], stroke);
        let y = r.top() + r.height() * frac;
        painter.line_segment([egui::pos2(r.left(), y), egui::pos2(r.right(), y)], stroke);
    }
}

fn draw_vehicle(
    painter: &egui::Painter,
    c: egui::Pos2,
    s: f32,
    color: Color32,
    stroke: egui::Stroke,
    steer: bool,
) {
    let body = egui::Rect::from_center_size(c, egui::vec2(s * 0.50, s * 0.30));
    painter.rect_stroke(body, 3.0, stroke, egui::StrokeKind::Middle);
    for x in [body.left(), body.right()] {
        for y in [body.top(), body.bottom()] {
            let p1 = egui::pos2(x, y);
            let p2 = if steer && x > c.x {
                egui::pos2(x + s * 0.05, y - s * 0.04)
            } else {
                egui::pos2(x, y)
            };
            painter.circle_filled(p2, 1.8, color);
            painter.line_segment([p1, p2], egui::Stroke::new(0.8, color));
        }
    }
}

fn draw_mecanum(
    painter: &egui::Painter,
    c: egui::Pos2,
    s: f32,
    color: Color32,
    stroke: egui::Stroke,
) {
    draw_vehicle(painter, c, s, color, stroke, false);
    for x in [-0.24, 0.24] {
        for y in [-0.16, 0.16] {
            let p = egui::pos2(c.x + s * x, c.y + s * y);
            painter.line_segment(
                [
                    egui::pos2(p.x - s * 0.04, p.y + s * 0.04),
                    egui::pos2(p.x + s * 0.04, p.y - s * 0.04),
                ],
                egui::Stroke::new(1.0, color),
            );
        }
    }
}

fn draw_omni(painter: &egui::Painter, c: egui::Pos2, s: f32, color: Color32, count: usize) {
    let radius = s * 0.20;
    for i in 0..count {
        let angle = i as f32 * std::f32::consts::TAU / count as f32;
        let p = egui::pos2(c.x + angle.cos() * radius, c.y + angle.sin() * radius);
        painter.circle_filled(p, 2.0, color);
        painter.line_segment([c, p], egui::Stroke::new(1.0, color));
    }
    painter.circle_stroke(c, s * 0.06, egui::Stroke::new(1.0, color));
}

fn draw_tracked(
    painter: &egui::Painter,
    c: egui::Pos2,
    s: f32,
    color: Color32,
    stroke: egui::Stroke,
) {
    for y in [-0.14, 0.14] {
        let r = egui::Rect::from_center_size(
            egui::pos2(c.x, c.y + s * y),
            egui::vec2(s * 0.56, s * 0.12),
        );
        painter.rect_stroke(r, 6.0, stroke, egui::StrokeKind::Middle);
        painter.circle_filled(egui::pos2(r.left() + s * 0.08, r.center().y), 1.2, color);
        painter.circle_filled(egui::pos2(r.right() - s * 0.08, r.center().y), 1.2, color);
    }
}

fn draw_arm(
    painter: &egui::Painter,
    c: egui::Pos2,
    s: f32,
    color: Color32,
    stroke: egui::Stroke,
    joints: usize,
) {
    let mut points = vec![egui::pos2(c.x - s * 0.24, c.y + s * 0.18)];
    points.push(egui::pos2(c.x - s * 0.08, c.y - s * 0.06));
    points.push(egui::pos2(c.x + s * 0.12, c.y - s * 0.16));
    if joints > 2 {
        points.push(egui::pos2(c.x + s * 0.24, c.y + s * 0.08));
    }
    for pair in points.windows(2) {
        painter.line_segment([pair[0], pair[1]], stroke);
    }
    for p in points {
        painter.circle_filled(p, 1.8, color);
    }
}

fn draw_delta(
    painter: &egui::Painter,
    c: egui::Pos2,
    s: f32,
    color: Color32,
    stroke: egui::Stroke,
) {
    let top = egui::pos2(c.x, c.y - s * 0.22);
    let left = egui::pos2(c.x - s * 0.22, c.y + s * 0.12);
    let right = egui::pos2(c.x + s * 0.22, c.y + s * 0.12);
    for pair in [[top, left], [top, right], [left, right]] {
        painter.line_segment(pair, stroke);
    }
    painter.circle_filled(c, 1.8, color);
    for p in [top, left, right] {
        painter.circle_filled(p, 1.6, color);
        painter.line_segment([p, c], egui::Stroke::new(0.8, color));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn icon_kind_round_trips_known_keys() {
        for key in [
            "dashboard",
            "connections",
            "terminal",
            "packet",
            "topology",
            "pid",
            "nn",
            "viz",
            "simulation",
            "line",
            "differential",
            "six_dof_arm",
        ] {
            assert_eq!(IconKind::from_key(key).key(), key);
        }
        assert_eq!(IconKind::from_key("missing"), IconKind::Generic);
    }

    #[test]
    fn runtime_source_has_no_emoji_icon_literals() {
        let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut offenders = Vec::new();
        scan_rs_files(&src, &mut offenders);
        assert!(
            offenders.is_empty(),
            "runtime source contains forbidden icon literals:\n{}",
            offenders.join("\n")
        );
    }

    fn scan_rs_files(path: &Path, offenders: &mut Vec<String>) {
        let entries = fs::read_dir(path).expect("read source directory");
        for entry in entries {
            let path = entry.expect("read source entry").path();
            if path.is_dir() {
                scan_rs_files(&path, offenders);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                let text = fs::read_to_string(&path).expect("read source file");
                for (line_idx, line) in text.lines().enumerate() {
                    if line.chars().any(is_forbidden_icon_char) {
                        offenders.push(format!("{}:{}", path.display(), line_idx + 1));
                    }
                }
            }
        }
    }

    fn is_forbidden_icon_char(ch: char) -> bool {
        matches!(
            ch as u32,
            0x1F300..=0x1FAFF | 0x2600..=0x27BF
        )
    }
}
