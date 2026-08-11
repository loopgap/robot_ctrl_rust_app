use egui::{self, Color32, FontFamily, FontId, RichText, TextStyle, Ui};

// ═══════════════════════════════════════════════════════════
// Design Token System
// ═══════════════════════════════════════════════════════════

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
    // ── Backgrounds ──────────────────────────────────────
    pub bg_dark: Color32,
    pub bg_medium: Color32,
    pub bg_card: Color32,
    pub bg_input: Color32,
    // ── Text hierarchy ───────────────────────────────────
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_muted: Color32,
    pub text_label: Color32,
    // ── Status ───────────────────────────────────────────
    pub status_ok: Color32,
    pub status_error: Color32,
    pub status_warn: Color32,
    pub status_info: Color32,
    // ── Accents ──────────────────────────────────────────
    pub accent_blue: Color32,
    pub accent_green: Color32,
    pub accent_purple: Color32,
    pub accent_orange: Color32,
    pub accent_cyan: Color32,
    pub accent_gold: Color32,
    // ── Borders ──────────────────────────────────────────
    pub border: Color32,
    pub border_active: Color32,
    // ── Direction (TX/RX/Info) ───────────────────────────
    pub tx_color: Color32,
    pub rx_color: Color32,
    pub info_color: Color32,
    // ── Connection status ────────────────────────────────
    pub connected_color: Color32,
    pub disconnected_color: Color32,
    // ── Semantic data colors ─────────────────────────────
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
    /// Dark theme — deep navy palette, WCAG AA compliant text.
    pub fn dark() -> Self {
        Self {
            bg_dark: Color32::from_rgb(22, 28, 38),
            bg_medium: Color32::from_rgb(30, 35, 42),
            bg_card: Color32::from_rgba_premultiplied(50, 50, 60, 180),
            bg_input: Color32::from_rgb(25, 50, 80),
            text_primary: Color32::from_rgb(220, 220, 230),
            text_secondary: Color32::from_rgb(200, 210, 220),
            text_muted: Color32::from_rgb(152, 162, 172),
            text_label: Color32::from_rgb(170, 180, 200),
            status_ok: Color32::from_rgb(46, 160, 67),
            status_error: Color32::from_rgb(255, 100, 100),
            status_warn: Color32::from_rgb(255, 180, 120),
            status_info: Color32::from_rgb(100, 200, 255),
            accent_blue: Color32::from_rgb(88, 166, 255),
            accent_green: Color32::from_rgb(0, 255, 160),
            accent_purple: Color32::from_rgb(200, 150, 255),
            accent_orange: Color32::from_rgb(255, 165, 0),
            accent_cyan: Color32::from_rgb(0, 200, 180),
            accent_gold: Color32::from_rgb(255, 200, 100),
            border: Color32::from_rgb(50, 60, 75),
            border_active: Color32::from_rgb(88, 166, 255),
            tx_color: Color32::from_rgb(120, 200, 255),
            rx_color: Color32::from_rgb(130, 230, 160),
            info_color: Color32::from_rgb(220, 220, 140),
            connected_color: Color32::from_rgb(46, 160, 67),
            disconnected_color: Color32::from_rgb(128, 128, 128),
            data_label: Color32::from_rgb(180, 180, 255),
            data_value: Color32::from_rgb(255, 200, 100),
            data_positive: Color32::from_rgb(120, 220, 120),
            data_negative: Color32::from_rgb(220, 100, 100),
        }
    }

    /// Light theme — soft warm palette, WCAG AA compliant text.
    pub fn light() -> Self {
        Self {
            bg_dark: Color32::from_rgb(240, 240, 245),
            bg_medium: Color32::from_rgb(230, 230, 235),
            bg_card: Color32::from_rgba_premultiplied(255, 255, 255, 230),
            bg_input: Color32::from_rgb(245, 248, 255),
            text_primary: Color32::from_rgb(30, 30, 40),
            text_secondary: Color32::from_rgb(60, 60, 70),
            text_muted: Color32::from_rgb(96, 96, 106),
            text_label: Color32::from_rgb(80, 80, 90),
            status_ok: Color32::from_rgb(40, 160, 60),
            status_error: Color32::from_rgb(220, 60, 60),
            status_warn: Color32::from_rgb(200, 140, 40),
            status_info: Color32::from_rgb(50, 130, 220),
            accent_blue: Color32::from_rgb(50, 120, 220),
            accent_green: Color32::from_rgb(0, 180, 120),
            accent_purple: Color32::from_rgb(150, 100, 220),
            accent_orange: Color32::from_rgb(220, 130, 0),
            accent_cyan: Color32::from_rgb(0, 150, 136),
            accent_gold: Color32::from_rgb(180, 130, 0),
            border: Color32::from_rgb(200, 200, 210),
            border_active: Color32::from_rgb(50, 120, 220),
            tx_color: Color32::from_rgb(50, 120, 200),
            rx_color: Color32::from_rgb(40, 150, 80),
            info_color: Color32::from_rgb(180, 160, 50),
            connected_color: Color32::from_rgb(40, 160, 60),
            disconnected_color: Color32::from_rgb(160, 160, 160),
            data_label: Color32::from_rgb(100, 100, 180),
            data_value: Color32::from_rgb(180, 120, 0),
            data_positive: Color32::from_rgb(30, 140, 50),
            data_negative: Color32::from_rgb(200, 50, 50),
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
fn luminance(c: Color32) -> f32 {
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

// ═══════════════════════════════════════════════════════════
// Reusable UI Components (available for view integration)
// ═══════════════════════════════════════════════════════════
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
    let c = rect.center();
    let s = rect.width().min(rect.height());
    let stroke = egui::Stroke::new(1.5_f32, color);
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
                egui::Stroke::new(1.0_f32, color),
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
                egui::Stroke::new(1.1_f32, color),
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
                    egui::Stroke::new(1.0_f32, color),
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
    painter.line_segment([n1, n3], egui::Stroke::new(1.0_f32, color));
    painter.line_segment([n2, n3], egui::Stroke::new(1.0_f32, color));
    painter.line_segment([n3, n4], egui::Stroke::new(1.0_f32, color));
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
        painter.line_segment([c, end], egui::Stroke::new(1.0_f32, color));
    }
    draw_line_chart(
        painter,
        egui::pos2(c.x, c.y + s * 0.04),
        s * 0.72,
        egui::Stroke::new(1.0_f32, color),
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
        egui::Stroke::new(1.0_f32, color),
    );
    painter.line_segment(
        [
            egui::pos2(r.left() + s * 0.22, r.top()),
            egui::pos2(r.left() + s * 0.22, r.bottom()),
        ],
        egui::Stroke::new(1.0_f32, color),
    );
}

fn draw_canopen(painter: &egui::Painter, c: egui::Pos2, s: f32, color: Color32) {
    let p1 = egui::pos2(c.x - s * 0.20, c.y);
    let p2 = egui::pos2(c.x, c.y - s * 0.18);
    let p3 = egui::pos2(c.x + s * 0.20, c.y);
    let p4 = egui::pos2(c.x, c.y + s * 0.18);
    for pair in [[p1, p2], [p2, p3], [p3, p4], [p4, p1]] {
        painter.line_segment(pair, egui::Stroke::new(1.1_f32, color));
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
        egui::Stroke::new(1.2_f32, color),
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
            painter.line_segment([p1, p2], egui::Stroke::new(0.8_f32, color));
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
                egui::Stroke::new(1.0_f32, color),
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
        painter.line_segment([c, p], egui::Stroke::new(1.0_f32, color));
    }
    painter.circle_stroke(c, s * 0.06, egui::Stroke::new(1.0_f32, color));
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
        painter.line_segment([p, c], egui::Stroke::new(0.8_f32, color));
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
