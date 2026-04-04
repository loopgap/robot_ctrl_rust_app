use eframe::egui;

pub const ACCENT_COLOR: egui::Color32 = egui::Color32::from_rgb(0, 122, 204);
pub const PANEL_BG: egui::Color32 = egui::Color32::from_rgb(30, 30, 30);
pub const PANEL_BG_LIGHT: egui::Color32 = egui::Color32::from_rgb(245, 247, 250);

pub fn apply_theme(ctx: &egui::Context, dark_mode: bool) {
    let mut visuals = if dark_mode {
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = PANEL_BG;
        visuals.override_text_color = Some(egui::Color32::from_rgb(220, 220, 230));
        visuals
    } else {
        let mut visuals = egui::Visuals::light();
        visuals.panel_fill = PANEL_BG_LIGHT;
        visuals
    };
    visuals.selection.bg_fill = ACCENT_COLOR.gamma_multiply(0.7);
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
    style.spacing.item_spacing = egui::vec2(10.0, 10.0);
    style.spacing.button_padding = egui::vec2(14.0, 8.0);
    style.spacing.window_margin = egui::Margin::same(16);
    style.spacing.interact_size.y = 32.0;
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
