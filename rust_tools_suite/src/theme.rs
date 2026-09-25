use eframe::egui;

pub const ACCENT_COLOR: egui::Color32 = egui::Color32::from_rgb(0, 122, 204);
pub const PANEL_BG: egui::Color32 = egui::Color32::from_rgb(15, 23, 42);
pub const PANEL_BG_LIGHT: egui::Color32 = egui::Color32::from_rgb(248, 250, 252);
pub const CARD_BG: egui::Color32 = egui::Color32::from_rgb(30, 41, 59);
pub const CARD_BG_LIGHT: egui::Color32 = egui::Color32::from_rgb(255, 255, 255);
pub const INPUT_BG: egui::Color32 = egui::Color32::from_rgb(19, 29, 45);
pub const INPUT_BG_LIGHT: egui::Color32 = egui::Color32::from_rgb(248, 250, 252);
pub const BORDER_COLOR: egui::Color32 = egui::Color32::from_rgb(51, 65, 85);
pub const BORDER_COLOR_LIGHT: egui::Color32 = egui::Color32::from_rgb(226, 232, 240);

pub fn apply_theme(ctx: &egui::Context, dark_mode: bool) {
    let mut visuals = if dark_mode {
        let mut v = egui::Visuals::dark();
        v.panel_fill = PANEL_BG;
        v.window_fill = CARD_BG;
        v.faint_bg_color = CARD_BG;
        v.extreme_bg_color = INPUT_BG;
        v.code_bg_color = INPUT_BG;
        v.override_text_color = Some(egui::Color32::from_rgb(241, 245, 249));
        v.window_stroke = egui::Stroke::new(1.0_f32, BORDER_COLOR);
        v.window_corner_radius = egui::CornerRadius::same(10);
        v.widgets.noninteractive.bg_fill = CARD_BG;
        v.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0_f32, BORDER_COLOR);
        v.widgets.noninteractive.corner_radius = egui::CornerRadius::same(6);
        v.widgets.inactive.bg_fill = CARD_BG;
        v.widgets.inactive.bg_stroke = egui::Stroke::new(1.0_f32, BORDER_COLOR);
        v.widgets.inactive.corner_radius = egui::CornerRadius::same(6);
        v.widgets.hovered.bg_stroke = egui::Stroke::new(1.2_f32, ACCENT_COLOR);
        v.widgets.hovered.corner_radius = egui::CornerRadius::same(6);
        v.widgets.active.bg_fill = ACCENT_COLOR.gamma_multiply(0.25);
        v.widgets.active.bg_stroke = egui::Stroke::new(1.5_f32, ACCENT_COLOR);
        v.widgets.active.corner_radius = egui::CornerRadius::same(6);
        v
    } else {
        let mut v = egui::Visuals::light();
        v.panel_fill = PANEL_BG_LIGHT;
        v.window_fill = CARD_BG_LIGHT;
        v.faint_bg_color = CARD_BG_LIGHT;
        v.extreme_bg_color = INPUT_BG_LIGHT;
        v.code_bg_color = INPUT_BG_LIGHT;
        v.override_text_color = Some(egui::Color32::from_rgb(15, 23, 42));
        v.window_stroke = egui::Stroke::new(1.0_f32, BORDER_COLOR_LIGHT);
        v.window_corner_radius = egui::CornerRadius::same(10);
        v.widgets.noninteractive.bg_fill = CARD_BG_LIGHT;
        v.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0_f32, BORDER_COLOR_LIGHT);
        v.widgets.noninteractive.corner_radius = egui::CornerRadius::same(6);
        v.widgets.inactive.bg_fill = CARD_BG_LIGHT;
        v.widgets.inactive.bg_stroke = egui::Stroke::new(1.0_f32, BORDER_COLOR_LIGHT);
        v.widgets.inactive.corner_radius = egui::CornerRadius::same(6);
        v.widgets.hovered.bg_stroke = egui::Stroke::new(1.2_f32, ACCENT_COLOR);
        v.widgets.hovered.corner_radius = egui::CornerRadius::same(6);
        v.widgets.active.bg_fill = ACCENT_COLOR.gamma_multiply(0.18);
        v.widgets.active.bg_stroke = egui::Stroke::new(1.5_f32, ACCENT_COLOR);
        v.widgets.active.corner_radius = egui::CornerRadius::same(6);
        v
    };
    visuals.selection.bg_fill = ACCENT_COLOR.gamma_multiply(0.35);
    visuals.selection.stroke = egui::Stroke::new(1.0_f32, ACCENT_COLOR);
    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.text_styles.insert(
        egui::TextStyle::Small,
        egui::FontId::new(13.5, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::new(15.5, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Button,
        egui::FontId::new(15.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Monospace,
        egui::FontId::new(14.5, egui::FontFamily::Monospace),
    );
    style.text_styles.insert(
        egui::TextStyle::Heading,
        egui::FontId::new(24.0, egui::FontFamily::Proportional),
    );
    style.spacing.item_spacing = egui::vec2(10.0, 8.0);
    style.spacing.button_padding = egui::vec2(12.0, 7.0);
    style.spacing.window_margin = egui::Margin::same(16);
    style.spacing.interact_size.y = 34.0;
    style.spacing.text_edit_width = 320.0;
    ctx.set_style(style);
}

fn try_load_cjk_font() -> Option<Vec<u8>> {
    let candidates: &[&str] = if cfg!(target_os = "windows") {
        &[
            r"C:\Windows\Fonts\msyh.ttc",
            r"C:\Windows\Fonts\msyhbd.ttc",
            r"C:\Windows\Fonts\msyhl.ttc",
            r"C:\Windows\Fonts\msyh.ttf",
            r"C:\Windows\Fonts\Deng.ttf",
            r"C:\Windows\Fonts\Dengb.ttf",
            r"C:\Windows\Fonts\simsun.ttc",
            r"C:\Windows\Fonts\simsunb.ttf",
            r"C:\Windows\Fonts\simhei.ttf",
        ]
    } else if cfg!(target_os = "macos") {
        &[
            "/System/Library/Fonts/PingFang.ttc",
            "/System/Library/Fonts/Hiragino Sans GB.ttc",
            "/System/Library/Fonts/STHeiti Medium.ttc",
        ]
    } else {
        &[
            "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
        ]
    };

    for path in candidates {
        if let Ok(data) = std::fs::read(path) {
            return Some(data);
        }
    }
    None
}

pub fn install_font_fallback(ctx: &egui::Context) {
    let Some(font_data) = try_load_cjk_font() else {
        return;
    };

    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "system-cjk".into(),
        egui::FontData::from_owned(font_data).into(),
    );

    if let Some(proportional) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
        proportional.insert(0, "system-cjk".into());
    }
    if let Some(monospace) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
        monospace.insert(0, "system-cjk".into());
    }

    ctx.set_fonts(fonts);
}
