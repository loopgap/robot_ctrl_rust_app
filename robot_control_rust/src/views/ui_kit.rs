use egui::{self, Color32, FontFamily, FontId, RichText, TextStyle, Ui};

// Design Token System

/// Spacing tokens based on 4px/8px grid system.
/// Provides 9 levels of consistent spacing across the UI.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct SpacingTokens {
    /// 4px — tight inline spacing, icon gaps
    pub xs: f32,
    /// 8px — base unit, small gaps
    pub sm: f32,
    /// 12px — medium gaps, card internal padding
    pub md: f32,
    /// 16px — standard item spacing, section gaps
    pub lg: f32,
    /// 20px — comfortable gaps
    pub xl: f32,
    /// 24px — heading margins, card outer spacing
    pub xxl: f32,
    /// 32px — section dividers
    pub xxxl: f32,
    /// 40px — page section spacing
    pub xxxxl: f32,
    /// 48px — major section breaks
    pub xxxxxl: f32,
}

impl SpacingTokens {
    /// Default spacing for standard density.
    pub fn standard() -> Self {
        Self {
            xs: 4.0,
            sm: 8.0,
            md: 12.0,
            lg: 16.0,
            xl: 20.0,
            xxl: 24.0,
            xxxl: 32.0,
            xxxxl: 40.0,
            xxxxxl: 48.0,
        }
    }

    /// Compact spacing for dense layouts (small screens).
    pub fn compact() -> Self {
        Self {
            xs: 2.0,
            sm: 4.0,
            md: 8.0,
            lg: 12.0,
            xl: 16.0,
            xxl: 20.0,
            xxxl: 24.0,
            xxxxl: 32.0,
            xxxxxl: 40.0,
        }
    }

    /// Relaxed spacing for wide layouts.
    pub fn relaxed() -> Self {
        Self {
            xs: 6.0,
            sm: 10.0,
            md: 16.0,
            lg: 20.0,
            xl: 28.0,
            xxl: 32.0,
            xxxl: 40.0,
            xxxxl: 52.0,
            xxxxxl: 64.0,
        }
    }
}

/// Responsive breakpoints for adaptive layout decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ResponsiveBreakpoint {
    /// < 640px — compact panels, stacked layout
    Compact,
    /// 640–1024px — standard desktop, side-by-side where possible
    Medium,
    /// > 1024px — wide layout, full feature display
    Wide,
}

impl ResponsiveBreakpoint {
    /// Determine breakpoint from available width.
    pub fn from_width(width: f32) -> Self {
        if width < 640.0 {
            Self::Compact
        } else if width < 1024.0 {
            Self::Medium
        } else {
            Self::Wide
        }
    }

    /// Get the matching spacing tokens for this breakpoint.
    pub fn spacing(self) -> SpacingTokens {
        match self {
            Self::Compact => SpacingTokens::compact(),
            Self::Medium => SpacingTokens::standard(),
            Self::Wide => SpacingTokens::relaxed(),
        }
    }

    /// Get the recommended interact size for this breakpoint.
    pub fn interact_size_y(self) -> f32 {
        match self {
            Self::Compact => 32.0,
            Self::Medium => 36.0,
            Self::Wide => 38.0,
        }
    }

    /// Whether to use a two-column layout for settings cards.
    #[allow(dead_code)]
    pub fn use_two_column(self) -> bool {
        matches!(self, Self::Wide)
    }
}

/// Animation duration tokens (in seconds).
/// Provides 5 levels of timing for consistent animation feel.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct DurationTokens {
    /// 100ms — instant micro-interactions (opacity, subtle shifts)
    pub instant: f32,
    /// 150ms — fast transitions (button states, toggles)
    pub fast: f32,
    /// 200ms — normal transitions (hover effects, color changes)
    pub normal: f32,
    /// 300ms — slow transitions (panels, modals entering)
    pub slow: f32,
    /// 500ms — emphasis transitions (page-level, loading states)
    pub slower: f32,
}

impl DurationTokens {
    pub fn standard() -> Self {
        Self {
            instant: 0.10,
            fast: 0.15,
            normal: 0.20,
            slow: 0.30,
            slower: 0.50,
        }
    }
}

/// Unified easing curves for the design system.
#[allow(dead_code)]
pub struct EasingTokens;

impl EasingTokens {
    /// Standard ease-out for most UI transitions.
    pub fn standard() -> crate::app::animation::Easing {
        crate::app::animation::Easing::EaseOutCubic
    }

    /// Emphasized ease-out for entrances and expansions.
    #[allow(dead_code)]
    pub fn emphasized() -> crate::app::animation::Easing {
        crate::app::animation::Easing::Bezier(0.05, 0.7, 0.1, 1.0)
    }

    /// Smooth ease-in-out for continuous animations.
    #[allow(dead_code)]
    pub fn smooth() -> crate::app::animation::Easing {
        crate::app::animation::Easing::EaseInOutCubic
    }
}

/// Font tokens — unified typography scale across all views.
///
/// All sizes are logical pixels at 100% scale. The `apply_to_style` method
/// installs them as egui `TextStyle` entries so built-in widgets pick them up.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FontTokens {
    /// Large display / page titles (dashboard hero).
    pub display: FontId,
    /// Section headings inside cards.
    pub heading: FontId,
    /// Sub-section titles (collapsible headers, card titles).
    pub subheading: FontId,
    /// Default body / paragraph text.
    pub body: FontId,
    /// Button labels and interactive controls.
    pub button: FontId,
    /// Small helper text, captions, timestamps.
    pub caption: FontId,
    /// Inline code, hex dumps, register tables.
    pub mono: FontId,
    /// Extra-large values (data-viz hero numbers).
    pub hero_value: FontId,
}

#[allow(dead_code)]
impl FontTokens {
    /// Default token set — tuned for a 15 px body baseline.
    pub fn default_tokens() -> Self {
        Self {
            display: FontId::new(26.0, FontFamily::Proportional),
            heading: FontId::new(22.0, FontFamily::Proportional),
            subheading: FontId::new(16.0, FontFamily::Proportional),
            body: FontId::new(15.0, FontFamily::Proportional),
            button: FontId::new(14.5, FontFamily::Proportional),
            caption: FontId::new(12.0, FontFamily::Proportional),
            mono: FontId::new(13.5, FontFamily::Monospace),
            hero_value: FontId::new(28.0, FontFamily::Proportional),
        }
    }

    /// Apply font tokens as egui `TextStyle` entries.
    pub fn apply_to_style(&self, style: &mut egui::Style) {
        style
            .text_styles
            .insert(TextStyle::Small, self.caption.clone());
        style.text_styles.insert(TextStyle::Body, self.body.clone());
        style
            .text_styles
            .insert(TextStyle::Button, self.button.clone());
        style
            .text_styles
            .insert(TextStyle::Monospace, self.mono.clone());
        style
            .text_styles
            .insert(TextStyle::Heading, self.heading.clone());
    }
}

/// Semantic color tokens for consistent UI/UX across all views.
///
/// 33 tokens organized by role: backgrounds, text hierarchy, status,
/// accents, borders, direction, and semantic data colors.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AppTheme {
    pub bg_dark: Color32,
    pub bg_medium: Color32,
    pub bg_card: Color32,
    pub bg_input: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_muted: Color32,
    pub text_label: Color32,
    pub status_ok: Color32,
    pub status_error: Color32,
    pub status_warn: Color32,
    pub status_info: Color32,
    pub accent_blue: Color32,
    pub accent_green: Color32,
    pub accent_purple: Color32,
    pub accent_orange: Color32,
    pub accent_cyan: Color32,
    pub accent_gold: Color32,
    pub border: Color32,
    pub border_active: Color32,
    pub tx_color: Color32,
    pub rx_color: Color32,
    pub info_color: Color32,
    pub connected_color: Color32,
    pub disconnected_color: Color32,
    /// Labels for structured data fields (table headers, type annotations)
    pub data_label: Color32,
    /// Numeric or value display for structured data
    pub data_value: Color32,
    /// Positive delta, pass, OK state
    pub data_positive: Color32,
    /// Negative delta, fail, error state
    pub data_negative: Color32,
}

impl AppTheme {
    /// Dark theme — deep slate-navy palette, WCAG AA compliant text.
    pub fn dark() -> Self {
        Self {
            bg_dark: Color32::from_rgb(15, 23, 42),
            bg_medium: Color32::from_rgb(24, 34, 53),
            bg_card: Color32::from_rgb(30, 41, 59),
            bg_input: Color32::from_rgb(19, 29, 45),
            text_primary: Color32::from_rgb(241, 245, 249),
            text_secondary: Color32::from_rgb(203, 213, 225),
            text_muted: Color32::from_rgb(148, 163, 184),
            text_label: Color32::from_rgb(175, 192, 215),
            status_ok: Color32::from_rgb(34, 197, 94),
            status_error: Color32::from_rgb(248, 113, 113),
            status_warn: Color32::from_rgb(251, 191, 36),
            status_info: Color32::from_rgb(56, 189, 248),
            accent_blue: Color32::from_rgb(59, 130, 246),
            accent_green: Color32::from_rgb(16, 185, 129),
            accent_purple: Color32::from_rgb(168, 85, 247),
            accent_orange: Color32::from_rgb(249, 115, 22),
            accent_cyan: Color32::from_rgb(6, 182, 212),
            accent_gold: Color32::from_rgb(234, 179, 8),
            border: Color32::from_rgb(51, 65, 85),
            border_active: Color32::from_rgb(56, 189, 248),
            tx_color: Color32::from_rgb(96, 165, 250),
            rx_color: Color32::from_rgb(52, 211, 153),
            info_color: Color32::from_rgb(250, 204, 21),
            connected_color: Color32::from_rgb(34, 197, 94),
            disconnected_color: Color32::from_rgb(100, 116, 139),
            data_label: Color32::from_rgb(165, 180, 252),
            data_value: Color32::from_rgb(253, 186, 116),
            data_positive: Color32::from_rgb(74, 222, 128),
            data_negative: Color32::from_rgb(248, 113, 113),
        }
    }

    /// Light theme — clean crisp slate palette, WCAG AA compliant text.
    pub fn light() -> Self {
        Self {
            bg_dark: Color32::from_rgb(248, 250, 252),
            bg_medium: Color32::from_rgb(241, 245, 249),
            bg_card: Color32::from_rgb(255, 255, 255),
            bg_input: Color32::from_rgb(248, 250, 252),
            text_primary: Color32::from_rgb(15, 23, 42),
            text_secondary: Color32::from_rgb(51, 65, 85),
            text_muted: Color32::from_rgb(100, 116, 139),
            text_label: Color32::from_rgb(71, 85, 105),
            status_ok: Color32::from_rgb(22, 163, 74),
            status_error: Color32::from_rgb(220, 38, 38),
            status_warn: Color32::from_rgb(217, 119, 6),
            status_info: Color32::from_rgb(2, 132, 199),
            accent_blue: Color32::from_rgb(37, 99, 235),
            accent_green: Color32::from_rgb(5, 150, 105),
            accent_purple: Color32::from_rgb(147, 51, 234),
            accent_orange: Color32::from_rgb(234, 88, 12),
            accent_cyan: Color32::from_rgb(8, 145, 178),
            accent_gold: Color32::from_rgb(202, 138, 4),
            border: Color32::from_rgb(226, 232, 240),
            border_active: Color32::from_rgb(37, 99, 235),
            tx_color: Color32::from_rgb(37, 99, 235),
            rx_color: Color32::from_rgb(22, 163, 74),
            info_color: Color32::from_rgb(202, 138, 4),
            connected_color: Color32::from_rgb(22, 163, 74),
            disconnected_color: Color32::from_rgb(148, 163, 184),
            data_label: Color32::from_rgb(99, 102, 241),
            data_value: Color32::from_rgb(217, 119, 6),
            data_positive: Color32::from_rgb(22, 163, 74),
            data_negative: Color32::from_rgb(220, 38, 38),
        }
    }

    /// High-contrast variant of the current theme.
    ///
    /// Boosts text to near-white/near-black and strengthens borders
    /// for WCAG AAA-level readability.
    #[allow(dead_code)]
    pub fn high_contrast(&self) -> Self {
        let is_dark = luminance(self.bg_dark) < 0.5;
        let mut t = self.clone();
        if is_dark {
            t.text_primary = Color32::from_rgb(255, 255, 255);
            t.text_secondary = Color32::from_rgb(230, 235, 240);
            t.text_muted = Color32::from_rgb(190, 200, 210);
            t.text_label = Color32::from_rgb(210, 220, 235);
            t.border = Color32::from_rgb(80, 95, 115);
            t.status_info = Color32::from_rgb(130, 220, 255);
            t.accent_blue = Color32::from_rgb(110, 185, 255);
            t.data_label = Color32::from_rgb(200, 200, 255);
        } else {
            t.text_primary = Color32::from_rgb(0, 0, 0);
            t.text_secondary = Color32::from_rgb(30, 30, 40);
            t.text_muted = Color32::from_rgb(60, 60, 70);
            t.text_label = Color32::from_rgb(40, 40, 50);
            t.border = Color32::from_rgb(160, 160, 170);
            t.status_info = Color32::from_rgb(30, 100, 190);
            t.accent_blue = Color32::from_rgb(30, 90, 190);
            t.data_label = Color32::from_rgb(70, 70, 150);
        }
        t
    }
}

/// Relative luminance (0.0–1.0) per WCAG definition.
pub fn luminance(c: Color32) -> f32 {
    fn lin(v: u8) -> f32 {
        let s = v as f32 / 255.0;
        if s <= 0.04045 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    }
    0.2126 * lin(c.r()) + 0.7152 * lin(c.g()) + 0.0722 * lin(c.b())
}

/// WCAG contrast ratio between two colors (1.0–21.0).
#[allow(dead_code)]
pub fn contrast_ratio(c1: Color32, c2: Color32) -> f32 {
    let l1 = luminance(c1);
    let l2 = luminance(c2);
    let lighter = l1.max(l2);
    let darker = l1.min(l2);
    (lighter + 0.05) / (darker + 0.05)
}

// Reusable UI Components (available for view integration)
#[allow(dead_code, clippy::too_many_arguments)]
/// Animated status badge with smooth color transition.
pub fn status_badge(
    ui: &mut Ui,
    anim: &mut crate::app::animation::AnimationManager,
    current_time: f64,
    key: &str,
    text: &str,
    ok: bool,
    theme: &AppTheme,
) {
    let target_color = if ok {
        theme.status_ok
    } else {
        theme.status_error
    };
    let durations = DurationTokens::standard();
    let color = anim.animate_color(
        format!("badge_{}", key),
        target_color,
        target_color,
        durations.normal,
        EasingTokens::standard(),
        current_time,
    );
    let tokens = FontTokens::default_tokens();
    ui.horizontal(|ui| {
        ui.colored_label(color, "●");
        ui.label(RichText::new(text).size(tokens.caption.size));
    });
}

/// Animated status dot (colored circle only).
#[allow(dead_code)]
pub fn status_dot(
    ui: &mut Ui,
    anim: &mut crate::app::animation::AnimationManager,
    current_time: f64,
    key: &str,
    ok: bool,
    theme: &AppTheme,
) {
    let target_color = if ok {
        theme.status_ok
    } else {
        theme.disconnected_color
    };
    let durations = DurationTokens::standard();
    let color = anim.animate_color(
        format!("dot_{}", key),
        target_color,
        target_color,
        durations.normal,
        EasingTokens::standard(),
        current_time,
    );
    ui.colored_label(color, "●");
}

/// Toast notification with auto-dismiss. Returns true if still visible.
#[allow(dead_code, clippy::too_many_arguments)]
pub fn toast(
    ui: &mut Ui,
    _anim: &mut crate::app::animation::AnimationManager,
    current_time: f64,
    message: &str,
    is_error: bool,
    start_time: f64,
    duration_secs: f64,
    theme: &AppTheme,
) -> bool {
    let elapsed = current_time - start_time;
    if elapsed > duration_secs + 0.5 {
        return false;
    }
    let alpha = if elapsed > duration_secs {
        ((duration_secs + 0.5 - elapsed) / 0.5).clamp(0.0, 1.0) as u8
    } else {
        255
    };
    let bg_color = if is_error {
        Color32::from_rgba_premultiplied(120, 30, 30, alpha)
    } else {
        Color32::from_rgba_premultiplied(30, 80, 40, alpha)
    };
    let text_color = if is_error {
        theme.status_error
    } else {
        theme.status_ok
    };
    egui::Frame::new()
        .fill(bg_color)
        .corner_radius(8.0)
        .inner_margin(egui::Margin::symmetric(16, 10))
        .show(ui, |ui| {
            let tokens = FontTokens::default_tokens();
            ui.horizontal(|ui| {
                let icon = if is_error { "ERR" } else { "OK" };
                ui.label(
                    RichText::new(icon)
                        .color(text_color)
                        .size(tokens.button.size),
                );
                ui.label(
                    RichText::new(message)
                        .color(Color32::WHITE)
                        .size(tokens.caption.size),
                );
            });
        });
    true
}

/// Pulsing loading spinner with text.
#[allow(dead_code)]
pub fn loading_spinner(
    ui: &mut Ui,
    _anim: &mut crate::app::animation::AnimationManager,
    current_time: f64,
    _key: &str,
    message: &str,
    theme: &AppTheme,
) {
    let tokens = FontTokens::default_tokens();
    let pulse = ((current_time * 3.0).sin() * 0.3 + 0.7).clamp(0.4, 1.0) as f32;
    let color = theme.accent_blue.linear_multiply(pulse);
    ui.horizontal(|ui| {
        ui.spinner();
        ui.label(
            RichText::new(message)
                .color(color)
                .size(tokens.caption.size),
        );
    });
}

/// Empty state placeholder when no data is available.
#[allow(dead_code)]
pub fn empty_state(ui: &mut Ui, icon: &str, title: &str, subtitle: &str, theme: &AppTheme) {
    let sp = SpacingTokens::standard();
    let tokens = FontTokens::default_tokens();
    ui.vertical_centered(|ui| {
        ui.add_space(sp.xxl);
        ui.label(
            RichText::new(icon)
                .size(tokens.hero_value.size)
                .color(theme.text_muted),
        );
        ui.add_space(sp.sm);
        ui.label(
            RichText::new(title)
                .size(tokens.body.size)
                .strong()
                .color(theme.text_secondary),
        );
        ui.add_space(sp.xs);
        ui.label(
            RichText::new(subtitle)
                .size(tokens.caption.size)
                .color(theme.text_muted),
        );
        ui.add_space(sp.xxl);
    });
}

/// Skeleton rectangular placeholder with physical shimmer wave animation for smooth data loading states.
#[allow(dead_code)]
pub fn skeleton_rect(
    ui: &mut Ui,
    size: egui::Vec2,
    corner_radius: f32,
    theme: &AppTheme,
    current_time: f64,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::hover());
    let painter = ui.painter();

    // Background placeholder fill
    painter.rect_filled(rect, corner_radius, theme.bg_input);

    // Smooth moving shimmer band
    let wave_speed = 1.25; // 1.25s per sweep
    let phase = ((current_time * wave_speed) % 1.0) as f32;
    let band_center = rect.left() + rect.width() * phase;
    let band_width = (rect.width() * 0.45).clamp(30.0, 160.0);

    let band_left = (band_center - band_width * 0.5).max(rect.left());
    let band_right = (band_center + band_width * 0.5).min(rect.right());
    if band_right > band_left {
        let shimmer_rect = egui::Rect::from_min_max(
            egui::pos2(band_left, rect.top()),
            egui::pos2(band_right, rect.bottom()),
        );
        let dist = (band_center - band_left.midpoint(band_right)).abs();
        let factor = (1.0 - (dist / (band_width * 0.5 + 1.0))).clamp(0.0, 1.0);
        let alpha = (28.0 * factor) as u8;
        let is_dark = luminance(theme.bg_dark) < 0.5;
        let shimmer_color = if is_dark {
            Color32::from_rgba_premultiplied(255, 255, 255, alpha)
        } else {
            Color32::from_rgba_premultiplied(0, 0, 0, (alpha / 2).max(1))
        };
        painter.rect_filled(shimmer_rect, corner_radius, shimmer_color);
    }

    // Subtle border
    painter.rect_stroke(
        rect,
        corner_radius,
        egui::Stroke::new(1.0_f32, theme.border),
        egui::StrokeKind::Middle,
    );
    ui.ctx().request_repaint();
    response
}

/// Skeleton lines placeholder mimicking text or multi-row record loading.
#[allow(dead_code)]
pub fn skeleton_lines(ui: &mut Ui, lines: usize, theme: &AppTheme, current_time: f64) {
    let avail_w = ui.available_width();
    let line_height = 14.0;
    let gap = 8.0;
    for i in 0..lines {
        let width_factor = match i % 3 {
            0 => 0.95,
            1 => 0.75,
            _ => 0.55,
        };
        let w = (avail_w * width_factor).max(60.0);
        skeleton_rect(ui, egui::vec2(w, line_height), 4.0, theme, current_time);
        if i + 1 < lines {
            ui.add_space(gap);
        }
    }
}

/// Skeleton card container placeholder for dashboard panels or settings cards.
#[allow(dead_code)]
pub fn skeleton_card(ui: &mut Ui, height: f32, theme: &AppTheme, current_time: f64) {
    let avail_w = ui.available_width();
    settings_card(ui, |ui| {
        skeleton_rect(
            ui,
            egui::vec2(avail_w.max(120.0), height),
            6.0,
            theme,
            current_time,
        );
    });
}

/// Styled button variants.
#[allow(dead_code)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Danger,
}

/// Styled button with color variant and hover/press feedback.
#[allow(dead_code)]
pub fn styled_button(
    ui: &mut Ui,
    label: &str,
    variant: ButtonVariant,
    theme: &AppTheme,
) -> egui::Response {
    let tokens = FontTokens::default_tokens();
    let (text_color, bg_color) = match variant {
        ButtonVariant::Primary => (Color32::WHITE, theme.accent_blue),
        ButtonVariant::Secondary => (theme.text_primary, theme.bg_medium),
        ButtonVariant::Danger => (Color32::WHITE, theme.status_error),
    };
    let btn = egui::Button::new(
        RichText::new(label)
            .color(text_color)
            .size(tokens.button.size),
    )
    .fill(bg_color)
    .min_size(egui::vec2(80.0, 30.0));
    let resp = ui.add(btn);
    // Apply subtle hover/press color shift feedback
    if resp.hovered() && !resp.is_pointer_button_down_on() {
        let painter = ui.painter();
        painter.rect_filled(
            resp.rect,
            4.0,
            Color32::from_rgba_premultiplied(255, 255, 255, 12),
        );
    } else if resp.is_pointer_button_down_on() {
        let painter = ui.painter();
        painter.rect_filled(
            resp.rect,
            4.0,
            Color32::from_rgba_premultiplied(0, 0, 0, 24),
        );
    }
    resp
}

/// Animated value display — smoothly transitions between values.
#[allow(dead_code)]
pub fn animated_value_text(
    ui: &mut Ui,
    anim: &mut crate::app::animation::AnimationManager,
    current_time: f64,
    key: &str,
    value: f64,
    format_fn: impl FnOnce(f64) -> String,
    theme: &AppTheme,
) {
    let durations = DurationTokens::standard();
    let smooth = anim.animate_float(
        key.to_string(),
        value as f32,
        value as f32,
        durations.slow,
        EasingTokens::standard(),
        current_time,
    );
    let tokens = FontTokens::default_tokens();
    let text = format_fn(smooth as f64);
    ui.label(
        RichText::new(text)
            .size(tokens.caption.size)
            .strong()
            .color(theme.text_primary),
    );
}

pub fn apply_page_style(ui: &mut Ui) {
    let bp = ResponsiveBreakpoint::from_width(ui.available_width());
    let sp = bp.spacing();
    let spacing = ui.spacing_mut();
    spacing.item_spacing = egui::vec2(sp.lg, sp.md);
    spacing.button_padding = egui::vec2(sp.md, sp.sm);
    spacing.interact_size.y = bp.interact_size_y();
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
    let sp = SpacingTokens::standard();
    let tokens = FontTokens::default_tokens();
    apply_page_style(ui);
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(22.0, 22.0), egui::Sense::hover());
        draw_icon(ui.painter(), rect, icon, ui.visuals().text_color());
        ui.add_space(sp.sm);
        ui.heading(RichText::new(title).size(tokens.display.size));
    });
    ui.add_space(sp.md);
}

pub fn section_title(ui: &mut Ui, text: &str) {
    let sp = SpacingTokens::standard();
    let tokens = FontTokens::default_tokens();
    ui.label(RichText::new(text).size(tokens.subheading.size).strong());
    ui.add_space(sp.sm);
}

pub fn settings_card(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) {
    let sp = SpacingTokens::standard();
    egui::Frame::group(ui.style())
        .fill(ui.visuals().faint_bg_color)
        .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
        .corner_radius(sp.md)
        .inner_margin(egui::Margin::symmetric(sp.lg as i8 + 2, sp.lg as i8))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            add_contents(ui);
        });
}

pub fn draw_icon(painter: &egui::Painter, rect: egui::Rect, icon: IconKind, color: Color32) {
    let s = rect.width().min(rect.height()).max(8.0);
    let ox = rect.center().x - s * 0.5;
    let oy = rect.center().y - s * 0.5;
    let scale = s / 24.0;
    let pt = |x: f32, y: f32| -> egui::Pos2 { egui::pos2(ox + x * scale, oy + y * scale) };

    let stroke = egui::Stroke::new((1.6 * scale).clamp(1.2, 2.4), color);
    let thin_stroke = egui::Stroke::new((1.1 * scale).clamp(0.9, 1.8), color);

    match icon {
        IconKind::Dashboard => {
            // Speedometer gauge arc + needle + tick marks
            let center = pt(12.0, 12.0);
            let r = 8.5 * scale;
            let mut pts = Vec::with_capacity(16);
            for i in 0..=12 {
                let angle =
                    std::f32::consts::PI * 0.75 + (i as f32 / 12.0) * std::f32::consts::PI * 1.5;
                pts.push(egui::pos2(
                    center.x + angle.cos() * r,
                    center.y + angle.sin() * r,
                ));
            }
            for i in 0..pts.len() - 1 {
                painter.line_segment([pts[i], pts[i + 1]], stroke);
            }
            painter.line_segment([center, pt(15.5, 8.5)], stroke);
            painter.circle_filled(center, 1.8 * scale, color);
            painter.line_segment([pt(5.5, 12.0), pt(7.2, 12.0)], thin_stroke);
            painter.line_segment([pt(12.0, 5.5), pt(12.0, 7.2)], thin_stroke);
            painter.line_segment([pt(16.8, 12.0), pt(18.5, 12.0)], thin_stroke);
        }
        IconKind::Connections => {
            // Dual socket & plug with data link
            let l_port = egui::Rect::from_min_max(pt(3.0, 8.0), pt(9.0, 16.0));
            let r_plug = egui::Rect::from_min_max(pt(15.0, 6.0), pt(21.0, 18.0));
            painter.rect_stroke(l_port, 2.0 * scale, stroke, egui::StrokeKind::Middle);
            painter.rect_stroke(r_plug, 2.0 * scale, stroke, egui::StrokeKind::Middle);
            painter.line_segment([pt(9.0, 10.5), pt(15.0, 10.5)], stroke);
            painter.line_segment([pt(9.0, 13.5), pt(15.0, 13.5)], stroke);
            painter.circle_filled(pt(6.0, 12.0), 1.2 * scale, color);
            painter.circle_filled(pt(18.0, 12.0), 1.5 * scale, color);
        }
        IconKind::Terminal => {
            // Terminal frame with prompt > and cursor _
            let term = egui::Rect::from_min_max(pt(3.0, 4.0), pt(21.0, 20.0));
            painter.rect_stroke(term, 2.5 * scale, stroke, egui::StrokeKind::Middle);
            painter.line_segment([pt(3.0, 8.0), pt(21.0, 8.0)], thin_stroke);
            painter.line_segment([pt(7.0, 11.0), pt(10.0, 13.5)], stroke);
            painter.line_segment([pt(10.0, 13.5), pt(7.0, 16.0)], stroke);
            painter.line_segment([pt(12.5, 16.0), pt(16.5, 16.0)], stroke);
        }
        IconKind::Packet => {
            // Segmented protocol packet frame
            let pkt = egui::Rect::from_min_max(pt(3.0, 4.0), pt(21.0, 20.0));
            painter.rect_stroke(pkt, 2.5 * scale, stroke, egui::StrokeKind::Middle);
            painter.line_segment([pt(3.0, 9.0), pt(21.0, 9.0)], stroke);
            painter.line_segment([pt(9.0, 9.0), pt(9.0, 20.0)], thin_stroke);
            painter.line_segment([pt(15.0, 9.0), pt(15.0, 20.0)], thin_stroke);
            painter.circle_filled(pt(6.0, 6.5), 1.1 * scale, color);
            painter.line_segment([pt(10.0, 6.5), pt(18.0, 6.5)], thin_stroke);
            painter.line_segment([pt(5.5, 13.0), pt(6.5, 13.0)], thin_stroke);
            painter.line_segment([pt(11.5, 14.5), pt(12.5, 14.5)], thin_stroke);
            painter.line_segment([pt(17.5, 14.5), pt(18.5, 14.5)], thin_stroke);
        }
        IconKind::Topology => {
            // Master hub with 4 peripheral node circles
            painter.circle_filled(pt(12.0, 12.0), 2.8 * scale, color);
            let satellites = [pt(5.0, 6.0), pt(19.0, 6.0), pt(5.0, 18.0), pt(19.0, 18.0)];
            for sat in satellites {
                painter.circle_stroke(sat, 2.0 * scale, stroke);
            }
            painter.line_segment([pt(6.8, 7.5), pt(9.8, 10.2)], stroke);
            painter.line_segment([pt(17.2, 7.5), pt(14.2, 10.2)], stroke);
            painter.line_segment([pt(6.8, 16.5), pt(9.8, 13.8)], stroke);
            painter.line_segment([pt(17.2, 16.5), pt(14.2, 13.8)], stroke);
        }
        IconKind::Pid => {
            // Step-response damped curve with dashed target setpoint line
            painter.line_segment([pt(3.0, 4.0), pt(3.0, 20.0)], stroke);
            painter.line_segment([pt(3.0, 20.0), pt(21.0, 20.0)], stroke);
            for i in 0..7 {
                painter.line_segment(
                    [
                        pt(3.0 + i as f32 * 2.5, 11.0),
                        pt(4.5 + i as f32 * 2.5, 11.0),
                    ],
                    thin_stroke,
                );
            }
            let pts = [
                pt(3.0, 19.5),
                pt(5.5, 14.0),
                pt(8.5, 7.0),
                pt(11.5, 13.5),
                pt(14.5, 9.5),
                pt(17.5, 11.5),
                pt(21.0, 11.0),
            ];
            for i in 0..pts.len() - 1 {
                painter.line_segment([pts[i], pts[i + 1]], stroke);
            }
        }
        IconKind::Neural => {
            // Neural network layer nodes with synapse weights
            let l1 = [pt(5.0, 6.0), pt(5.0, 12.0), pt(5.0, 18.0)];
            let l2 = [pt(12.0, 8.5), pt(12.0, 15.5)];
            let l3 = pt(19.0, 12.0);
            for n1 in l1 {
                for n2 in l2 {
                    painter.line_segment([n1, n2], thin_stroke);
                }
            }
            for n2 in l2 {
                painter.line_segment([n2, l3], thin_stroke);
            }
            for n1 in l1 {
                painter.circle_stroke(n1, 1.8 * scale, stroke);
            }
            for n2 in l2 {
                painter.circle_filled(n2, 1.8 * scale, color);
            }
            painter.circle_stroke(l3, 1.8 * scale, stroke);
        }
        IconKind::Visualization | IconKind::Line => {
            // Cartesian axes with trend curve
            painter.line_segment([pt(3.0, 4.0), pt(3.0, 20.0)], stroke);
            painter.line_segment([pt(3.0, 20.0), pt(21.0, 20.0)], stroke);
            let curve = [
                pt(4.0, 16.0),
                pt(8.5, 10.0),
                pt(12.5, 14.0),
                pt(17.0, 6.0),
                pt(20.5, 10.0),
            ];
            for i in 0..curve.len() - 1 {
                painter.line_segment([curve[i], curve[i + 1]], stroke);
            }
            painter.circle_filled(curve[1], 1.5 * scale, color);
            painter.circle_filled(curve[2], 1.5 * scale, color);
            painter.circle_filled(curve[3], 1.5 * scale, color);
        }
        IconKind::Simulation => {
            // Central rotor with magnetic orbits
            painter.circle_filled(pt(12.0, 12.0), 2.5 * scale, color);
            let r_x = 8.5 * scale;
            let r_y = 3.8 * scale;
            for rot in [-0.52_f32, 0.52_f32] {
                let mut pts = Vec::with_capacity(16);
                for i in 0..=12 {
                    let a = (i as f32 / 12.0) * std::f32::consts::TAU;
                    let x0 = a.cos() * r_x;
                    let y0 = a.sin() * r_y;
                    let x1 = x0 * rot.cos() - y0 * rot.sin();
                    let y1 = x0 * rot.sin() + y0 * rot.cos();
                    pts.push(egui::pos2(
                        ox + (12.0 + x1 / scale) * scale,
                        oy + (12.0 + y1 / scale) * scale,
                    ));
                }
                for i in 0..pts.len() - 1 {
                    painter.line_segment([pts[i], pts[i + 1]], stroke);
                }
            }
        }
        IconKind::Modbus => {
            // Stacked register blocks with bidirectional data transfer arrows
            let top_b = egui::Rect::from_min_max(pt(4.0, 4.0), pt(20.0, 10.5));
            let bot_b = egui::Rect::from_min_max(pt(4.0, 13.5), pt(20.0, 20.0));
            painter.rect_stroke(top_b, 2.0 * scale, stroke, egui::StrokeKind::Middle);
            painter.rect_stroke(bot_b, 2.0 * scale, stroke, egui::StrokeKind::Middle);
            painter.line_segment([pt(8.0, 7.25), pt(16.0, 7.25)], thin_stroke);
            painter.line_segment([pt(8.0, 16.75), pt(16.0, 16.75)], thin_stroke);
            // Down arrow
            painter.line_segment([pt(10.0, 10.5), pt(10.0, 13.5)], stroke);
            painter.line_segment([pt(8.5, 12.0), pt(10.0, 13.5)], thin_stroke);
            painter.line_segment([pt(11.5, 12.0), pt(10.0, 13.5)], thin_stroke);
            // Up arrow
            painter.line_segment([pt(14.0, 13.5), pt(14.0, 10.5)], stroke);
            painter.line_segment([pt(12.5, 12.0), pt(14.0, 10.5)], thin_stroke);
            painter.line_segment([pt(15.5, 12.0), pt(14.0, 10.5)], thin_stroke);
        }
        IconKind::Canopen => {
            // CAN transceivers linked on differential bus
            let n1 = egui::Rect::from_min_max(pt(3.0, 6.0), pt(9.0, 11.0));
            let n2 = egui::Rect::from_min_max(pt(15.0, 6.0), pt(21.0, 11.0));
            let n3 = egui::Rect::from_min_max(pt(9.0, 13.0), pt(15.0, 18.0));
            painter.rect_stroke(n1, 1.5 * scale, stroke, egui::StrokeKind::Middle);
            painter.rect_stroke(n2, 1.5 * scale, stroke, egui::StrokeKind::Middle);
            painter.rect_stroke(n3, 1.5 * scale, stroke, egui::StrokeKind::Middle);
            painter.line_segment([pt(6.0, 11.0), pt(6.0, 16.0)], stroke);
            painter.line_segment([pt(6.0, 16.0), pt(9.0, 16.0)], stroke);
            painter.line_segment([pt(18.0, 11.0), pt(18.0, 16.0)], stroke);
            painter.line_segment([pt(18.0, 16.0), pt(15.0, 16.0)], stroke);
            painter.line_segment([pt(12.0, 13.0), pt(12.0, 9.5)], stroke);
        }
        IconKind::Scatter => {
            painter.line_segment([pt(3.0, 4.0), pt(3.0, 20.0)], stroke);
            painter.line_segment([pt(3.0, 20.0), pt(21.0, 20.0)], stroke);
            for (x, y) in [(7.0, 15.0), (10.5, 9.0), (14.5, 13.0), (18.0, 7.5)] {
                painter.circle_filled(pt(x, y), 1.8 * scale, color);
            }
        }
        IconKind::Bar => {
            painter.line_segment([pt(3.0, 20.0), pt(21.0, 20.0)], stroke);
            let bars = [(5.5, 14.0), (10.0, 8.5), (14.5, 12.0), (19.0, 6.0)];
            for (x, y) in bars {
                let r = egui::Rect::from_min_max(pt(x - 1.5, y), pt(x + 1.5, 20.0));
                painter.rect_filled(r, 1.0 * scale, color);
            }
        }
        IconKind::Gauge => {
            painter.circle_stroke(pt(12.0, 13.0), 7.5 * scale, stroke);
            painter.line_segment([pt(12.0, 13.0), pt(16.5, 8.5)], stroke);
            painter.circle_filled(pt(12.0, 13.0), 1.8 * scale, color);
        }
        IconKind::Histogram => {
            painter.line_segment([pt(3.0, 20.0), pt(21.0, 20.0)], stroke);
            let bars = [
                (4.5, 16.0),
                (7.5, 12.0),
                (10.5, 7.0),
                (13.5, 9.5),
                (16.5, 14.0),
                (19.5, 17.5),
            ];
            for (x, y) in bars {
                let r = egui::Rect::from_min_max(pt(x - 1.2, y), pt(x + 1.2, 20.0));
                painter.rect_filled(r, 0.8 * scale, color);
            }
        }
        IconKind::Table => {
            let tbl = egui::Rect::from_min_max(pt(3.0, 4.0), pt(21.0, 20.0));
            painter.rect_stroke(tbl, 2.0 * scale, stroke, egui::StrokeKind::Middle);
            painter.line_segment([pt(3.0, 9.0), pt(21.0, 9.0)], stroke);
            painter.line_segment([pt(3.0, 14.5), pt(21.0, 14.5)], thin_stroke);
            painter.line_segment([pt(9.0, 4.0), pt(9.0, 20.0)], thin_stroke);
            painter.line_segment([pt(15.0, 4.0), pt(15.0, 20.0)], thin_stroke);
        }
        IconKind::Differential => {
            let body = egui::Rect::from_min_max(pt(6.5, 5.0), pt(17.5, 19.0));
            painter.rect_stroke(body, 2.5 * scale, stroke, egui::StrokeKind::Middle);
            let l_wheel = egui::Rect::from_min_max(pt(3.0, 10.0), pt(6.0, 17.0));
            let r_wheel = egui::Rect::from_min_max(pt(18.0, 10.0), pt(21.0, 17.0));
            painter.rect_filled(l_wheel, 1.2 * scale, color);
            painter.rect_filled(r_wheel, 1.2 * scale, color);
            painter.circle_filled(pt(12.0, 7.5), 1.8 * scale, color);
        }
        IconKind::Mecanum => {
            let body = egui::Rect::from_min_max(pt(6.5, 5.0), pt(17.5, 19.0));
            painter.rect_stroke(body, 2.0 * scale, stroke, egui::StrokeKind::Middle);
            let w1 = egui::Rect::from_min_max(pt(3.0, 5.5), pt(6.0, 10.5));
            let w2 = egui::Rect::from_min_max(pt(18.0, 5.5), pt(21.0, 10.5));
            let w3 = egui::Rect::from_min_max(pt(3.0, 13.5), pt(6.0, 18.5));
            let w4 = egui::Rect::from_min_max(pt(18.0, 13.5), pt(21.0, 18.5));
            for w in [w1, w2, w3, w4] {
                painter.rect_stroke(w, 1.0 * scale, stroke, egui::StrokeKind::Middle);
            }
            painter.line_segment([pt(3.5, 6.5), pt(5.5, 9.5)], thin_stroke);
            painter.line_segment([pt(18.5, 6.5), pt(20.5, 9.5)], thin_stroke);
            painter.line_segment([pt(3.5, 17.5), pt(5.5, 14.5)], thin_stroke);
            painter.line_segment([pt(18.5, 17.5), pt(20.5, 14.5)], thin_stroke);
        }
        IconKind::Omni3 => {
            painter.line_segment([pt(12.0, 4.0), pt(4.0, 18.0)], stroke);
            painter.line_segment([pt(4.0, 18.0), pt(20.0, 18.0)], stroke);
            painter.line_segment([pt(20.0, 18.0), pt(12.0, 4.0)], stroke);
            painter.rect_filled(
                egui::Rect::from_min_max(pt(10.5, 2.0), pt(13.5, 6.0)),
                1.0 * scale,
                color,
            );
            painter.rect_filled(
                egui::Rect::from_min_max(pt(2.5, 16.0), pt(5.5, 20.0)),
                1.0 * scale,
                color,
            );
            painter.rect_filled(
                egui::Rect::from_min_max(pt(18.5, 16.0), pt(21.5, 20.0)),
                1.0 * scale,
                color,
            );
            painter.circle_filled(pt(12.0, 13.0), 1.8 * scale, color);
        }
        IconKind::Omni4 => {
            let body = egui::Rect::from_min_max(pt(8.0, 8.0), pt(16.0, 16.0));
            painter.rect_stroke(body, 2.0 * scale, stroke, egui::StrokeKind::Middle);
            painter.rect_filled(
                egui::Rect::from_min_max(pt(10.5, 3.0), pt(13.5, 7.5)),
                1.0 * scale,
                color,
            );
            painter.rect_filled(
                egui::Rect::from_min_max(pt(10.5, 16.5), pt(13.5, 21.0)),
                1.0 * scale,
                color,
            );
            painter.rect_filled(
                egui::Rect::from_min_max(pt(3.0, 10.5), pt(7.5, 13.5)),
                1.0 * scale,
                color,
            );
            painter.rect_filled(
                egui::Rect::from_min_max(pt(16.5, 10.5), pt(21.0, 13.5)),
                1.0 * scale,
                color,
            );
            painter.circle_filled(pt(12.0, 12.0), 1.5 * scale, color);
        }
        IconKind::Ackermann => {
            let body = egui::Rect::from_min_max(pt(7.0, 5.0), pt(17.0, 19.0));
            painter.rect_stroke(body, 2.0 * scale, stroke, egui::StrokeKind::Middle);
            // Angled front wheels
            painter.line_segment([pt(5.0, 4.5), pt(3.5, 9.5)], stroke);
            painter.line_segment([pt(19.0, 4.5), pt(17.5, 9.5)], stroke);
            painter.line_segment([pt(4.25, 7.0), pt(18.25, 7.0)], thin_stroke);
            // Rear wheels
            let lr = egui::Rect::from_min_max(pt(3.0, 14.0), pt(6.0, 19.0));
            let rr = egui::Rect::from_min_max(pt(18.0, 14.0), pt(21.0, 19.0));
            painter.rect_filled(lr, 1.0 * scale, color);
            painter.rect_filled(rr, 1.0 * scale, color);
            painter.line_segment([pt(4.5, 16.5), pt(19.5, 16.5)], thin_stroke);
        }
        IconKind::Tracked => {
            let l_track = egui::Rect::from_min_max(pt(2.5, 4.0), pt(7.5, 20.0));
            let r_track = egui::Rect::from_min_max(pt(16.5, 4.0), pt(21.5, 20.0));
            let body = egui::Rect::from_min_max(pt(7.5, 7.0), pt(16.5, 17.0));
            painter.rect_stroke(l_track, 2.5 * scale, stroke, egui::StrokeKind::Middle);
            painter.rect_stroke(r_track, 2.5 * scale, stroke, egui::StrokeKind::Middle);
            painter.rect_stroke(body, 2.0 * scale, stroke, egui::StrokeKind::Middle);
            for y in [7.0, 12.0, 17.0] {
                painter.circle_filled(pt(5.0, y), 1.0 * scale, color);
                painter.circle_filled(pt(19.0, y), 1.0 * scale, color);
            }
        }
        IconKind::Scara => {
            let base = egui::Rect::from_min_max(pt(4.0, 17.0), pt(10.0, 21.0));
            painter.rect_stroke(base, 1.0 * scale, stroke, egui::StrokeKind::Middle);
            painter.circle_filled(pt(7.0, 15.0), 2.0 * scale, color);
            painter.line_segment([pt(7.0, 15.0), pt(13.0, 10.0)], stroke);
            painter.circle_filled(pt(13.0, 10.0), 2.0 * scale, color);
            painter.line_segment([pt(13.0, 10.0), pt(18.0, 12.0)], stroke);
            painter.circle_stroke(pt(18.0, 12.0), 1.5 * scale, stroke);
            painter.line_segment([pt(18.0, 13.5), pt(18.0, 19.5)], stroke);
            painter.line_segment([pt(16.5, 19.5), pt(19.5, 19.5)], stroke);
        }
        IconKind::SixDofArm => {
            let base = egui::Rect::from_min_max(pt(4.0, 18.0), pt(12.0, 21.0));
            painter.rect_stroke(base, 1.0 * scale, stroke, egui::StrokeKind::Middle);
            painter.circle_filled(pt(8.0, 17.0), 2.0 * scale, color);
            painter.line_segment([pt(8.0, 17.0), pt(10.0, 11.0)], stroke);
            painter.circle_filled(pt(10.0, 11.0), 1.8 * scale, color);
            painter.line_segment([pt(10.0, 11.0), pt(15.0, 8.0)], stroke);
            painter.circle_filled(pt(15.0, 8.0), 1.6 * scale, color);
            painter.line_segment([pt(15.0, 8.0), pt(18.0, 10.0)], stroke);
            painter.circle_stroke(pt(18.0, 10.0), 1.4 * scale, stroke);
            // Gripper
            painter.line_segment([pt(18.0, 10.0), pt(20.0, 12.0)], thin_stroke);
            painter.line_segment([pt(19.0, 13.0), pt(21.0, 12.0)], stroke);
            painter.line_segment([pt(21.0, 14.0), pt(20.0, 12.0)], stroke);
        }
        IconKind::DeltaRobot => {
            painter.line_segment([pt(5.0, 5.0), pt(19.0, 5.0)], stroke);
            painter.circle_filled(pt(6.0, 5.0), 1.4 * scale, color);
            painter.circle_filled(pt(12.0, 5.0), 1.4 * scale, color);
            painter.circle_filled(pt(18.0, 5.0), 1.4 * scale, color);
            painter.line_segment([pt(6.0, 6.5), pt(9.5, 12.0)], stroke);
            painter.line_segment([pt(9.5, 12.0), pt(8.5, 17.0)], stroke);
            painter.line_segment([pt(12.0, 6.5), pt(11.0, 12.0)], stroke);
            painter.line_segment([pt(11.0, 12.0), pt(12.0, 17.0)], stroke);
            painter.line_segment([pt(18.0, 6.5), pt(14.5, 12.0)], stroke);
            painter.line_segment([pt(14.5, 12.0), pt(15.5, 17.0)], stroke);
            painter.line_segment([pt(8.5, 17.0), pt(15.5, 17.0)], stroke);
            painter.line_segment([pt(12.0, 17.0), pt(12.0, 20.0)], stroke);
        }
        IconKind::Custom | IconKind::Generic => {
            let r = egui::Rect::from_min_max(pt(4.0, 4.0), pt(20.0, 20.0));
            painter.rect_stroke(r, 3.0 * scale, stroke, egui::StrokeKind::Middle);
            painter.line_segment([pt(8.0, 12.0), pt(16.0, 12.0)], thin_stroke);
            painter.line_segment([pt(12.0, 8.0), pt(12.0, 16.0)], thin_stroke);
            painter.circle_filled(pt(12.0, 12.0), 1.6 * scale, color);
        }
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
    #[test]
    fn spacing_tokens_standard_grid() {
        let sp = SpacingTokens::standard();
        assert_eq!(sp.xs, 4.0);
        assert_eq!(sp.sm, 8.0);
        assert_eq!(sp.md, 12.0);
        assert_eq!(sp.lg, 16.0);
        assert_eq!(sp.xl, 20.0);
        assert_eq!(sp.xxl, 24.0);
        assert_eq!(sp.xxxl, 32.0);
        assert_eq!(sp.xxxxl, 40.0);
        assert_eq!(sp.xxxxxl, 48.0);
    }

    #[test]
    fn spacing_tokens_compact_smaller_than_standard() {
        let s = SpacingTokens::standard();
        let c = SpacingTokens::compact();
        assert!(c.xs < s.xs);
        assert!(c.sm < s.sm);
        assert!(c.lg < s.lg);
        assert!(c.xxxxxl < s.xxxxxl);
    }

    #[test]
    fn spacing_tokens_relaxed_larger_than_standard() {
        let s = SpacingTokens::standard();
        let r = SpacingTokens::relaxed();
        assert!(r.xs > s.xs);
        assert!(r.sm > s.sm);
        assert!(r.lg > s.lg);
        assert!(r.xxxxxl > s.xxxxxl);
    }

    #[test]
    fn spacing_tokens_monotonic_ordering() {
        for sp in [
            SpacingTokens::standard(),
            SpacingTokens::compact(),
            SpacingTokens::relaxed(),
        ] {
            assert!(sp.xs <= sp.sm);
            assert!(sp.sm <= sp.md);
            assert!(sp.md <= sp.lg);
            assert!(sp.lg <= sp.xl);
            assert!(sp.xl <= sp.xxl);
            assert!(sp.xxl <= sp.xxxl);
            assert!(sp.xxxl <= sp.xxxxl);
            assert!(sp.xxxxl <= sp.xxxxxl);
        }
    }
    #[test]
    fn responsive_breakpoint_compact_below_640() {
        assert_eq!(
            ResponsiveBreakpoint::from_width(0.0),
            ResponsiveBreakpoint::Compact
        );
        assert_eq!(
            ResponsiveBreakpoint::from_width(320.0),
            ResponsiveBreakpoint::Compact
        );
        assert_eq!(
            ResponsiveBreakpoint::from_width(639.9),
            ResponsiveBreakpoint::Compact
        );
    }

    #[test]
    fn responsive_breakpoint_medium_640_to_1024() {
        assert_eq!(
            ResponsiveBreakpoint::from_width(640.0),
            ResponsiveBreakpoint::Medium
        );
        assert_eq!(
            ResponsiveBreakpoint::from_width(800.0),
            ResponsiveBreakpoint::Medium
        );
        assert_eq!(
            ResponsiveBreakpoint::from_width(1023.9),
            ResponsiveBreakpoint::Medium
        );
    }

    #[test]
    fn responsive_breakpoint_wide_above_1024() {
        assert_eq!(
            ResponsiveBreakpoint::from_width(1024.0),
            ResponsiveBreakpoint::Wide
        );
        assert_eq!(
            ResponsiveBreakpoint::from_width(1920.0),
            ResponsiveBreakpoint::Wide
        );
        assert_eq!(
            ResponsiveBreakpoint::from_width(4000.0),
            ResponsiveBreakpoint::Wide
        );
    }

    #[test]
    fn responsive_breakpoint_spacing_matches_tokens() {
        assert_eq!(
            ResponsiveBreakpoint::Compact.spacing().xs,
            SpacingTokens::compact().xs
        );
        assert_eq!(
            ResponsiveBreakpoint::Medium.spacing().xs,
            SpacingTokens::standard().xs
        );
        assert_eq!(
            ResponsiveBreakpoint::Wide.spacing().xs,
            SpacingTokens::relaxed().xs
        );
    }

    #[test]
    fn responsive_breakpoint_interact_size_y_values() {
        assert_eq!(ResponsiveBreakpoint::Compact.interact_size_y(), 32.0);
        assert_eq!(ResponsiveBreakpoint::Medium.interact_size_y(), 36.0);
        assert_eq!(ResponsiveBreakpoint::Wide.interact_size_y(), 38.0);
    }

    #[test]
    fn responsive_breakpoint_two_column_only_wide() {
        assert!(!ResponsiveBreakpoint::Compact.use_two_column());
        assert!(!ResponsiveBreakpoint::Medium.use_two_column());
        assert!(ResponsiveBreakpoint::Wide.use_two_column());
    }
    #[test]
    fn duration_tokens_standard_ordering() {
        let d = DurationTokens::standard();
        assert!(d.instant < d.fast);
        assert!(d.fast < d.normal);
        assert!(d.normal < d.slow);
        assert!(d.slow < d.slower);
    }

    #[test]
    fn duration_tokens_standard_values() {
        let d = DurationTokens::standard();
        assert!((d.instant - 0.10).abs() < f32::EPSILON);
        assert!((d.fast - 0.15).abs() < f32::EPSILON);
        assert!((d.normal - 0.20).abs() < f32::EPSILON);
        assert!((d.slow - 0.30).abs() < f32::EPSILON);
        assert!((d.slower - 0.50).abs() < f32::EPSILON);
    }
    #[test]
    fn app_theme_dark_and_light_not_identical() {
        let dark = AppTheme::dark();
        let light = AppTheme::light();
        assert_ne!(dark.bg_dark, light.bg_dark);
        assert_ne!(dark.text_primary, light.text_primary);
        assert_ne!(dark.status_ok, light.status_ok);
    }

    #[test]
    fn app_theme_high_contrast_dark_text_brighter() {
        let dark = AppTheme::dark();
        let hc = dark.high_contrast();
        // high-contrast dark theme pushes text toward white
        assert!(hc.text_primary.r() >= dark.text_primary.r());
        assert!(hc.text_primary.g() >= dark.text_primary.g());
        assert!(hc.text_primary.b() >= dark.text_primary.b());
    }

    #[test]
    fn app_theme_high_contrast_light_text_darker() {
        let light = AppTheme::light();
        let hc = light.high_contrast();
        // high-contrast light theme pushes text toward black
        assert!(hc.text_primary.r() <= light.text_primary.r());
        assert!(hc.text_primary.g() <= light.text_primary.g());
        assert!(hc.text_primary.b() <= light.text_primary.b());
    }
    #[test]
    fn luminance_pure_black_is_zero() {
        let l = luminance(Color32::from_rgb(0, 0, 0));
        assert!(l.abs() < 0.001);
    }

    #[test]
    fn luminance_pure_white_is_one() {
        let l = luminance(Color32::from_rgb(255, 255, 255));
        assert!((l - 1.0).abs() < 0.01);
    }

    #[test]
    fn luminance_monotonic_green_dominant() {
        // green has highest luminance coefficient
        let l_r = luminance(Color32::from_rgb(255, 0, 0));
        let l_g = luminance(Color32::from_rgb(0, 255, 0));
        let l_b = luminance(Color32::from_rgb(0, 0, 255));
        assert!(l_g > l_r);
        assert!(l_g > l_b);
    }

    #[test]
    fn contrast_ratio_same_color_is_one() {
        let c = Color32::from_rgb(128, 128, 128);
        let ratio = contrast_ratio(c, c);
        assert!((ratio - 1.0).abs() < 0.001);
    }

    #[test]
    fn contrast_ratio_black_white_is_21() {
        let ratio = contrast_ratio(Color32::from_rgb(0, 0, 0), Color32::from_rgb(255, 255, 255));
        assert!((ratio - 21.0).abs() < 0.5);
    }

    #[test]
    fn contrast_ratio_symmetric() {
        let a = Color32::from_rgb(100, 100, 100);
        let b = Color32::from_rgb(200, 200, 200);
        assert!((contrast_ratio(a, b) - contrast_ratio(b, a)).abs() < 0.001);
    }
    #[test]
    fn font_tokens_default_sizes_ordered() {
        let ft = FontTokens::default_tokens();
        // display > heading > subheading > body > button > caption
        assert!(ft.display.size > ft.heading.size);
        assert!(ft.heading.size > ft.subheading.size);
        assert!(ft.subheading.size > ft.body.size);
        assert!(ft.body.size > ft.button.size);
        assert!(ft.button.size > ft.caption.size);
    }

    #[test]
    fn font_tokens_mono_is_monospace() {
        let ft = FontTokens::default_tokens();
        assert_eq!(ft.mono.family, FontFamily::Monospace);
    }

    #[test]
    fn font_tokens_proportional_for_text() {
        let ft = FontTokens::default_tokens();
        assert_eq!(ft.body.family, FontFamily::Proportional);
        assert_eq!(ft.heading.family, FontFamily::Proportional);
        assert_eq!(ft.display.family, FontFamily::Proportional);
    }

    #[test]
    fn font_tokens_hero_larger_than_display() {
        let ft = FontTokens::default_tokens();
        assert!(ft.hero_value.size > ft.display.size);
    }
    #[test]
    fn icon_kind_all_keys_round_trip() {
        let all_keys = [
            "dashboard",
            "connections",
            "terminal",
            "packet",
            "topology",
            "pid",
            "nn",
            "viz",
            "simulation",
            "modbus",
            "canopen",
            "line",
            "scatter",
            "bar",
            "gauge",
            "histogram",
            "table",
            "differential",
            "mecanum",
            "omni3",
            "omni4",
            "ackermann",
            "tracked",
            "scara",
            "six_dof_arm",
            "delta_robot",
            "custom",
        ];
        for key in &all_keys {
            let icon = IconKind::from_key(key);
            assert_eq!(icon.key(), *key, "key '{}' round-trip failed", key);
        }
    }

    #[test]
    fn icon_kind_unknown_keys_map_to_generic() {
        for bad in &["", "unknown", "TYPO", "123", "line_"] {
            assert_eq!(
                IconKind::from_key(bad),
                IconKind::Generic,
                "key '{}' should be Generic",
                bad
            );
        }
    }
    #[test]
    fn button_variant_variants_exist() {
        // Verify all variants compile and are distinct
        assert!(matches!(ButtonVariant::Primary, ButtonVariant::Primary));
        assert!(matches!(ButtonVariant::Secondary, ButtonVariant::Secondary));
        assert!(matches!(ButtonVariant::Danger, ButtonVariant::Danger));
    }
    #[test]
    fn easing_tokens_return_valid_easings() {
        assert_eq!(
            EasingTokens::standard(),
            crate::app::animation::Easing::EaseOutCubic
        );
        assert_eq!(
            EasingTokens::emphasized(),
            crate::app::animation::Easing::Bezier(0.05, 0.7, 0.1, 1.0)
        );
        assert_eq!(
            EasingTokens::smooth(),
            crate::app::animation::Easing::EaseInOutCubic
        );
    }
    #[test]
    fn dark_theme_backgrounds_darker_than_light() {
        let dark = AppTheme::dark();
        let light = AppTheme::light();
        assert!(luminance(dark.bg_dark) < luminance(light.bg_dark));
        assert!(luminance(dark.bg_medium) < luminance(light.bg_medium));
    }

    #[test]
    fn theme_status_colors_distinct() {
        for theme in [AppTheme::dark(), AppTheme::light()] {
            assert_ne!(theme.status_ok, theme.status_error);
            assert_ne!(theme.status_ok, theme.status_warn);
            assert_ne!(theme.status_error, theme.status_warn);
            assert_ne!(theme.status_info, theme.status_error);
        }
    }

    #[test]
    fn theme_connected_disconnected_distinct() {
        for theme in [AppTheme::dark(), AppTheme::light()] {
            assert_ne!(theme.connected_color, theme.disconnected_color);
        }
    }
}
