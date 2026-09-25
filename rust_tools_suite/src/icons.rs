use eframe::egui::{self, Color32, Pos2, Rect, Stroke, StrokeKind};

/// Unified vector icon categories for `rust_tools_suite`.
/// Matches the 24x24 optical grid and stroke metrics of `robot_control_rust`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum ToolIconKind {
    SuiteBrand,
    At32Boot,
    Checksum,
    Json,
    Log,
    UrlCodec,
    TimeConverter,
    Base64,
    UuidBatch,
    CsvCleaner,
    JwtInspector,
    RegexWorkbench,
    Workflow,
    Settings,
    Docs,
    Shortcuts,
    Language,
    Export,
    Copy,
    Clear,
    Generic,
}

#[allow(dead_code)]
impl ToolIconKind {
    pub fn from_key(key: &str) -> Self {
        match key {
            "suite_brand" | "brand" => Self::SuiteBrand,
            "at32_boot_entry" | "at32_boot" | "boot" => Self::At32Boot,
            "checksum" => Self::Checksum,
            "json" => Self::Json,
            "log" => Self::Log,
            "url_codec" | "url" => Self::UrlCodec,
            "time_converter" | "time" => Self::TimeConverter,
            "base64" => Self::Base64,
            "uuid_batch" | "uuid" => Self::UuidBatch,
            "csv_cleaner" | "csv" => Self::CsvCleaner,
            "jwt_inspector" | "jwt" => Self::JwtInspector,
            "regex_workbench" | "regex" => Self::RegexWorkbench,
            "workflow" => Self::Workflow,
            "settings" => Self::Settings,
            "docs" => Self::Docs,
            "shortcuts" => Self::Shortcuts,
            "language" => Self::Language,
            "export" => Self::Export,
            "copy" => Self::Copy,
            "clear" => Self::Clear,
            _ => Self::Generic,
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Self::SuiteBrand => "suite_brand",
            Self::At32Boot => "at32_boot_entry",
            Self::Checksum => "checksum",
            Self::Json => "json",
            Self::Log => "log",
            Self::UrlCodec => "url_codec",
            Self::TimeConverter => "time_converter",
            Self::Base64 => "base64",
            Self::UuidBatch => "uuid_batch",
            Self::CsvCleaner => "csv_cleaner",
            Self::JwtInspector => "jwt_inspector",
            Self::RegexWorkbench => "regex_workbench",
            Self::Workflow => "workflow",
            Self::Settings => "settings",
            Self::Docs => "docs",
            Self::Shortcuts => "shortcuts",
            Self::Language => "language",
            Self::Export => "export",
            Self::Copy => "copy",
            Self::Clear => "clear",
            Self::Generic => "generic",
        }
    }
}

/// Draw a unified 24x24 vector icon into `rect` using `painter`.
/// All coordinates are normalized from [0, 24] to `rect` bounds, guaranteeing
/// resolution-independent sharpness and consistency across UI scales.
pub fn draw_tool_icon(painter: &egui::Painter, rect: Rect, icon: ToolIconKind, color: Color32) {
    let s = rect.width().min(rect.height()).max(8.0);
    let ox = rect.center().x - s * 0.5;
    let oy = rect.center().y - s * 0.5;
    let scale = s / 24.0;
    let pt = |x: f32, y: f32| -> Pos2 { egui::pos2(ox + x * scale, oy + y * scale) };

    let stroke = Stroke::new((1.6 * scale).clamp(1.2, 2.4), color);
    let thin_stroke = Stroke::new((1.1 * scale).clamp(0.9, 1.8), color);

    match icon {
        ToolIconKind::SuiteBrand => {
            // Hexagonal toolbox frame with central wrench handle
            let r = Rect::from_min_max(pt(4.0, 4.0), pt(20.0, 20.0));
            painter.rect_stroke(r, 4.0 * scale, stroke, StrokeKind::Middle);
            painter.line_segment([pt(8.0, 4.0), pt(8.0, 2.0)], stroke);
            painter.line_segment([pt(16.0, 4.0), pt(16.0, 2.0)], stroke);
            painter.line_segment([pt(8.0, 2.0), pt(16.0, 2.0)], stroke);
            painter.line_segment([pt(4.0, 11.0), pt(20.0, 11.0)], thin_stroke);
            painter.circle_filled(pt(12.0, 11.0), 1.6 * scale, color);
            painter.line_segment([pt(12.0, 12.6), pt(12.0, 16.5)], stroke);
        }
        ToolIconKind::At32Boot => {
            // Microcontroller QFP chip with 4-side pins and bootloader downward arrow
            let chip = Rect::from_min_max(pt(5.5, 5.5), pt(18.5, 18.5));
            painter.rect_stroke(chip, 2.0 * scale, stroke, StrokeKind::Middle);
            // Pins
            for i in [8.5, 12.0, 15.5] {
                painter.line_segment([pt(i, 2.5), pt(i, 5.5)], thin_stroke);
                painter.line_segment([pt(i, 18.5), pt(i, 21.5)], thin_stroke);
                painter.line_segment([pt(2.5, i), pt(5.5, i)], thin_stroke);
                painter.line_segment([pt(18.5, i), pt(21.5, i)], thin_stroke);
            }
            // Central boot arrow
            painter.line_segment([pt(12.0, 9.0), pt(12.0, 15.0)], stroke);
            painter.line_segment([pt(9.5, 12.5), pt(12.0, 15.0)], stroke);
            painter.line_segment([pt(14.5, 12.5), pt(12.0, 15.0)], stroke);
        }
        ToolIconKind::Checksum => {
            // Security shield with CRC checkmark
            let pts = [
                pt(12.0, 3.0),
                pt(4.5, 6.5),
                pt(4.5, 12.0),
                pt(12.0, 21.0),
                pt(19.5, 12.0),
                pt(19.5, 6.5),
                pt(12.0, 3.0),
            ];
            for i in 0..pts.len() - 1 {
                painter.line_segment([pts[i], pts[i + 1]], stroke);
            }
            // Checkmark
            painter.line_segment([pt(9.0, 12.0), pt(11.2, 14.5)], stroke);
            painter.line_segment([pt(11.2, 14.5), pt(15.5, 9.5)], stroke);
        }
        ToolIconKind::Json => {
            // Nested brackets { } with central data colon
            // Left brace {
            painter.line_segment([pt(8.5, 4.5), pt(6.5, 4.5)], stroke);
            painter.line_segment([pt(6.5, 4.5), pt(6.5, 10.0)], stroke);
            painter.line_segment([pt(6.5, 10.0), pt(4.5, 12.0)], stroke);
            painter.line_segment([pt(4.5, 12.0), pt(6.5, 14.0)], stroke);
            painter.line_segment([pt(6.5, 14.0), pt(6.5, 19.5)], stroke);
            painter.line_segment([pt(6.5, 19.5), pt(8.5, 19.5)], stroke);
            // Right brace }
            painter.line_segment([pt(15.5, 4.5), pt(17.5, 4.5)], stroke);
            painter.line_segment([pt(17.5, 4.5), pt(17.5, 10.0)], stroke);
            painter.line_segment([pt(17.5, 10.0), pt(19.5, 12.0)], stroke);
            painter.line_segment([pt(19.5, 12.0), pt(17.5, 14.0)], stroke);
            painter.line_segment([pt(17.5, 14.0), pt(17.5, 19.5)], stroke);
            painter.line_segment([pt(17.5, 19.5), pt(15.5, 19.5)], stroke);
            // Data nodes
            painter.circle_filled(pt(10.5, 12.0), 1.2 * scale, color);
            painter.circle_filled(pt(13.5, 12.0), 1.2 * scale, color);
        }
        ToolIconKind::Log => {
            // Document with log lines and magnifying glass
            let doc = [pt(14.0, 3.0), pt(5.0, 3.0), pt(5.0, 21.0), pt(13.0, 21.0)];
            painter.line_segment([doc[0], doc[1]], stroke);
            painter.line_segment([doc[1], doc[2]], stroke);
            painter.line_segment([doc[2], doc[3]], stroke);
            painter.line_segment([pt(14.0, 3.0), pt(19.0, 8.0)], thin_stroke);
            painter.line_segment([pt(19.0, 8.0), pt(14.0, 8.0)], thin_stroke);
            painter.line_segment([pt(14.0, 8.0), pt(14.0, 3.0)], thin_stroke);
            // Log lines
            painter.line_segment([pt(8.0, 9.0), pt(12.0, 9.0)], thin_stroke);
            painter.line_segment([pt(8.0, 13.0), pt(13.0, 13.0)], thin_stroke);
            // Lens
            painter.circle_stroke(pt(16.5, 16.5), 2.8 * scale, stroke);
            painter.line_segment([pt(18.5, 18.5), pt(21.0, 21.0)], stroke);
        }
        ToolIconKind::UrlCodec => {
            // Two interlocking chain links
            painter.circle_stroke(pt(9.0, 13.0), 4.2 * scale, stroke);
            painter.circle_stroke(pt(15.0, 11.0), 4.2 * scale, stroke);
            painter.line_segment([pt(11.5, 12.0), pt(12.5, 12.0)], stroke);
        }
        ToolIconKind::TimeConverter => {
            // Round clock face with hour/minute hands
            painter.circle_stroke(pt(12.0, 12.0), 8.5 * scale, stroke);
            painter.line_segment([pt(12.0, 7.0), pt(12.0, 12.0)], stroke);
            painter.line_segment([pt(12.0, 12.0), pt(15.5, 14.0)], stroke);
            painter.circle_filled(pt(12.0, 12.0), 1.4 * scale, color);
            // Top stopwatch button
            painter.line_segment([pt(10.5, 1.5), pt(13.5, 1.5)], thin_stroke);
        }
        ToolIconKind::Base64 => {
            // Left data column (binary 01) -> right data column (ASCII Aa)
            let l_box = Rect::from_min_max(pt(3.5, 4.5), pt(10.5, 19.5));
            let r_box = Rect::from_min_max(pt(13.5, 4.5), pt(20.5, 19.5));
            painter.rect_stroke(l_box, 2.0 * scale, stroke, StrokeKind::Middle);
            painter.rect_stroke(r_box, 2.0 * scale, stroke, StrokeKind::Middle);
            // Left bits
            painter.line_segment([pt(5.5, 8.5), pt(8.5, 8.5)], thin_stroke);
            painter.line_segment([pt(5.5, 12.0), pt(8.5, 12.0)], thin_stroke);
            painter.line_segment([pt(5.5, 15.5), pt(8.5, 15.5)], thin_stroke);
            // Right text
            painter.line_segment([pt(15.5, 8.5), pt(18.5, 8.5)], thin_stroke);
            painter.line_segment([pt(15.5, 12.0), pt(18.5, 12.0)], thin_stroke);
            painter.line_segment([pt(15.5, 15.5), pt(18.5, 15.5)], thin_stroke);
            // Arrow indicator
            painter.circle_filled(pt(12.0, 12.0), 1.2 * scale, color);
        }
        ToolIconKind::UuidBatch => {
            // Hexadecimal token tag with random bit matrix
            let tag = Rect::from_min_max(pt(3.5, 6.0), pt(20.5, 18.0));
            painter.rect_stroke(tag, 3.0 * scale, stroke, StrokeKind::Middle);
            painter.line_segment([pt(7.0, 10.0), pt(7.0, 14.0)], stroke);
            painter.line_segment([pt(10.0, 10.0), pt(10.0, 14.0)], stroke);
            painter.circle_filled(pt(13.5, 12.0), 1.4 * scale, color);
            painter.circle_filled(pt(17.0, 12.0), 1.4 * scale, color);
        }
        ToolIconKind::CsvCleaner => {
            // Spreadsheet grid with sparkle cleaner
            let grid = Rect::from_min_max(pt(3.5, 4.5), pt(20.5, 19.5));
            painter.rect_stroke(grid, 2.0 * scale, stroke, StrokeKind::Middle);
            painter.line_segment([pt(3.5, 10.0), pt(20.5, 10.0)], thin_stroke);
            painter.line_segment([pt(3.5, 15.0), pt(20.5, 15.0)], thin_stroke);
            painter.line_segment([pt(9.0, 4.5), pt(9.0, 19.5)], thin_stroke);
            painter.line_segment([pt(15.0, 4.5), pt(15.0, 19.5)], thin_stroke);
            // Sparkle
            painter.circle_filled(pt(17.5, 7.2), 1.5 * scale, color);
        }
        ToolIconKind::JwtInspector => {
            // Shield divided into 3 horizontal segments
            let pts = [
                pt(12.0, 2.5),
                pt(4.5, 5.5),
                pt(4.5, 12.0),
                pt(12.0, 21.0),
                pt(19.5, 12.0),
                pt(19.5, 5.5),
                pt(12.0, 2.5),
            ];
            for i in 0..pts.len() - 1 {
                painter.line_segment([pts[i], pts[i + 1]], stroke);
            }
            painter.line_segment([pt(5.5, 8.5), pt(18.5, 8.5)], thin_stroke);
            painter.line_segment([pt(6.5, 13.0), pt(17.5, 13.0)], thin_stroke);
            painter.circle_filled(pt(12.0, 10.8), 1.3 * scale, color);
        }
        ToolIconKind::RegexWorkbench => {
            // Code angle brackets < > with center asterisk *
            painter.line_segment([pt(7.0, 8.0), pt(4.0, 12.0)], stroke);
            painter.line_segment([pt(4.0, 12.0), pt(7.0, 16.0)], stroke);
            painter.line_segment([pt(17.0, 8.0), pt(20.0, 12.0)], stroke);
            painter.line_segment([pt(20.0, 12.0), pt(17.0, 16.0)], stroke);
            // Center *
            painter.line_segment([pt(12.0, 8.5), pt(12.0, 15.5)], stroke);
            painter.line_segment([pt(9.5, 10.2), pt(14.5, 13.8)], stroke);
            painter.line_segment([pt(14.5, 10.2), pt(9.5, 13.8)], stroke);
        }
        ToolIconKind::Workflow => {
            // Three interconnected pipeline workflow nodes
            painter.circle_stroke(pt(6.0, 6.0), 2.5 * scale, stroke);
            painter.circle_stroke(pt(18.0, 12.0), 2.5 * scale, stroke);
            painter.circle_stroke(pt(6.0, 18.0), 2.5 * scale, stroke);
            painter.line_segment([pt(8.5, 6.8), pt(15.5, 10.8)], thin_stroke);
            painter.line_segment([pt(15.5, 13.2), pt(8.5, 17.2)], thin_stroke);
        }
        ToolIconKind::Settings => {
            // Gear wheel
            painter.circle_stroke(pt(12.0, 12.0), 3.0 * scale, stroke);
            let outer_r = 7.5 * scale;
            let inner_r = 5.2 * scale;
            for i in 0..8 {
                let angle = i as f32 * std::f32::consts::TAU / 8.0;
                let c = pt(12.0, 12.0);
                let p1 = egui::pos2(c.x + angle.cos() * inner_r, c.y + angle.sin() * inner_r);
                let p2 = egui::pos2(c.x + angle.cos() * outer_r, c.y + angle.sin() * outer_r);
                painter.line_segment([p1, p2], stroke);
            }
        }
        ToolIconKind::Docs => {
            // Book with opened pages
            painter.line_segment([pt(12.0, 6.0), pt(12.0, 20.0)], stroke);
            painter.line_segment([pt(12.0, 6.0), pt(4.0, 4.0)], stroke);
            painter.line_segment([pt(4.0, 4.0), pt(4.0, 18.0)], stroke);
            painter.line_segment([pt(4.0, 18.0), pt(12.0, 20.0)], stroke);
            painter.line_segment([pt(12.0, 6.0), pt(20.0, 4.0)], stroke);
            painter.line_segment([pt(20.0, 4.0), pt(20.0, 18.0)], stroke);
            painter.line_segment([pt(20.0, 18.0), pt(12.0, 20.0)], stroke);
        }
        ToolIconKind::Shortcuts => {
            // Keyboard key with text
            let key = Rect::from_min_max(pt(3.5, 5.0), pt(20.5, 19.0));
            painter.rect_stroke(key, 3.0 * scale, stroke, StrokeKind::Middle);
            painter.line_segment([pt(7.0, 12.0), pt(17.0, 12.0)], thin_stroke);
        }
        ToolIconKind::Language => {
            // Globe with latitude and longitude arcs
            painter.circle_stroke(pt(12.0, 12.0), 8.5 * scale, stroke);
            painter.line_segment([pt(3.5, 12.0), pt(20.5, 12.0)], thin_stroke);
            painter.line_segment([pt(12.0, 3.5), pt(12.0, 20.5)], thin_stroke);
        }
        ToolIconKind::Export => {
            // Tray with exit arrow
            painter.line_segment([pt(4.0, 14.0), pt(4.0, 19.0)], stroke);
            painter.line_segment([pt(4.0, 19.0), pt(20.0, 19.0)], stroke);
            painter.line_segment([pt(20.0, 19.0), pt(20.0, 14.0)], stroke);
            painter.line_segment([pt(12.0, 4.0), pt(12.0, 14.0)], stroke);
            painter.line_segment([pt(8.5, 7.5), pt(12.0, 4.0)], stroke);
            painter.line_segment([pt(15.5, 7.5), pt(12.0, 4.0)], stroke);
        }
        ToolIconKind::Copy => {
            // Double overlapping sheets
            let back = Rect::from_min_max(pt(8.0, 4.0), pt(19.0, 15.0));
            let front = Rect::from_min_max(pt(5.0, 8.0), pt(16.0, 19.0));
            painter.rect_stroke(back, 2.0 * scale, thin_stroke, StrokeKind::Middle);
            painter.rect_stroke(front, 2.0 * scale, stroke, StrokeKind::Middle);
        }
        ToolIconKind::Clear => {
            // Trash can with lid
            painter.line_segment([pt(4.0, 6.0), pt(20.0, 6.0)], stroke);
            painter.line_segment([pt(9.0, 6.0), pt(9.0, 3.5)], thin_stroke);
            painter.line_segment([pt(9.0, 3.5), pt(15.0, 3.5)], thin_stroke);
            painter.line_segment([pt(15.0, 3.5), pt(15.0, 6.0)], thin_stroke);
            let can = Rect::from_min_max(pt(6.0, 6.0), pt(18.0, 20.0));
            painter.rect_stroke(can, 2.0 * scale, stroke, StrokeKind::Middle);
            painter.line_segment([pt(10.0, 10.0), pt(10.0, 16.0)], thin_stroke);
            painter.line_segment([pt(14.0, 10.0), pt(14.0, 16.0)], thin_stroke);
        }
        ToolIconKind::Generic => {
            // Bounded square with center crosshair
            let r = Rect::from_min_max(pt(5.0, 5.0), pt(19.0, 19.0));
            painter.rect_stroke(r, 3.0 * scale, stroke, StrokeKind::Middle);
            painter.line_segment([pt(9.0, 12.0), pt(15.0, 12.0)], thin_stroke);
            painter.line_segment([pt(12.0, 9.0), pt(12.0, 15.0)], thin_stroke);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_icon_keys_round_trip() {
        let keys = [
            "suite_brand",
            "at32_boot_entry",
            "checksum",
            "json",
            "log",
            "url_codec",
            "time_converter",
            "base64",
            "uuid_batch",
            "csv_cleaner",
            "jwt_inspector",
            "regex_workbench",
            "workflow",
            "settings",
            "docs",
            "shortcuts",
            "language",
            "export",
            "copy",
            "clear",
        ];
        for k in keys {
            let icon = ToolIconKind::from_key(k);
            assert_eq!(icon.key(), k, "Key '{}' failed round-trip", k);
        }
    }

    #[test]
    fn test_tool_icon_unknown_key_falls_back_to_generic() {
        assert_eq!(ToolIconKind::from_key("unknown_xyz"), ToolIconKind::Generic);
        assert_eq!(ToolIconKind::from_key(""), ToolIconKind::Generic);
    }
}
