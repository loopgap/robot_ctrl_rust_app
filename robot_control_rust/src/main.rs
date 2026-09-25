#![windows_subsystem = "windows"]

#[allow(dead_code)]
mod app;
mod i18n;
#[allow(dead_code)]
mod models;
#[allow(dead_code)]
mod services;
mod views;

use app::{
    ActiveTab, AppState, DisplayMode, LogDirection, DEFAULT_UI_SCALE_PERCENT, MAX_UI_SCALE_PERCENT,
    MIN_UI_SCALE_PERCENT, UI_SCALE_STEP_PERCENT,
};
use eframe::egui::{self, RichText};
use i18n::{Language, Tr};
use std::path::PathBuf;
use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::{Duration, Instant};

struct NavGroup {
    name_zh: &'static str,
    name_en: &'static str,
    tabs: &'static [ActiveTab],
}

const NAV_GROUPS: &[NavGroup] = &[
    NavGroup {
        name_zh: "基础与通信",
        name_en: "CORE & COMM",
        tabs: &[
            ActiveTab::Dashboard,
            ActiveTab::Connections,
            ActiveTab::SerialDebug,
        ],
    },
    NavGroup {
        name_zh: "协议与总线",
        name_en: "BUS & PROTOCOL",
        tabs: &[
            ActiveTab::ProtocolAnalysis,
            ActiveTab::PacketBuilder,
            ActiveTab::ModbusTools,
            ActiveTab::CanopenTools,
            ActiveTab::Topology,
        ],
    },
    NavGroup {
        name_zh: "控制与算法",
        name_en: "CONTROL & AI",
        tabs: &[
            ActiveTab::PidControl,
            ActiveTab::NnTuning,
            ActiveTab::DataViz,
            ActiveTab::SimulationLab,
        ],
    },
];

struct RobotControlApp {
    state: AppState,
    pending_ui_scale_percent: u32,
    last_prefs_save: Instant,
    last_saved_prefs_snapshot: Option<String>,
    prefs_save_tx: Sender<(PathBuf, String)>,
    applied_dark_mode: Option<bool>,
    applied_ui_scale_percent: Option<u32>,
    show_preferences: bool,
    show_about: bool,
    show_shortcuts: bool,
    top_tab_mode: bool,
    previous_tab: ActiveTab,
    tab_transition_start: Option<f64>,
    sidebar_actual_width: f32,
}

impl RobotControlApp {
    fn new() -> Self {
        let state = AppState::new();
        let pending_ui_scale_percent = state.ui.ui_scale_percent;
        let last_saved_prefs_snapshot = state.preferences_snapshot().ok().map(|(_, text)| text);
        let (prefs_save_tx, prefs_save_rx) = mpsc::channel::<(PathBuf, String)>();
        thread::Builder::new()
            .name("prefs-save-worker".into())
            .spawn(move || {
                while let Ok((path, text)) = prefs_save_rx.recv() {
                    let _ = AppState::write_preferences_snapshot(&path, &text);
                }
            })
            .expect("spawn prefs-save-worker");
        let initial_tab = state.active_tab;
        let initial_sidebar_width = if state.ui.sidebar_expanded {
            190.0
        } else {
            54.0
        };
        Self {
            state,
            pending_ui_scale_percent,
            last_prefs_save: Instant::now(),
            last_saved_prefs_snapshot,
            prefs_save_tx,
            applied_dark_mode: None,
            applied_ui_scale_percent: None,
            show_preferences: false,
            show_about: false,
            show_shortcuts: false,
            top_tab_mode: false,
            previous_tab: initial_tab,
            tab_transition_start: None,
            sidebar_actual_width: initial_sidebar_width,
        }
    }

    fn repaint_interval_ms(&self) -> u64 {
        self.state.repaint_interval_ms()
    }

    fn effective_repaint_interval_ms(&self, ctx: &egui::Context) -> u64 {
        let (minimized, focused) = ctx.input(|i| (i.viewport().minimized, i.viewport().focused));
        let mut interval = self.repaint_interval_ms();

        if minimized.unwrap_or(false) {
            interval = interval.max(500);
        } else if !focused.unwrap_or(true) {
            interval = interval.max(125);
        }

        interval
    }

    fn motion_level_label(lang: Language, idx: usize) -> &'static str {
        match idx {
            0 => Tr::motion_level_extreme(lang),
            1 => Tr::motion_level_standard(lang),
            2 => Tr::motion_level_native(lang),
            _ => Tr::motion_level_optimized(lang),
        }
    }

    fn apply_theme(&self, ctx: &egui::Context) {
        let theme = &self.state.theme;
        let is_dark = self.state.dark_mode;
        let mut visuals = if is_dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };

        visuals.override_text_color = Some(theme.text_primary);
        visuals.panel_fill = theme.bg_dark;
        visuals.window_fill = theme.bg_medium;
        visuals.faint_bg_color = theme.bg_card;
        visuals.extreme_bg_color = theme.bg_input;
        visuals.code_bg_color = theme.bg_input;
        visuals.window_stroke = egui::Stroke::new(1.0_f32, theme.border);
        visuals.window_corner_radius = egui::CornerRadius::same(10);

        visuals.selection.bg_fill = theme.accent_blue.gamma_multiply(0.35);
        visuals.selection.stroke = egui::Stroke::new(1.0_f32, theme.accent_blue);

        // Non-interactive widgets (frames, separators, labels)
        visuals.widgets.noninteractive.bg_fill = theme.bg_card;
        visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0_f32, theme.border);
        visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0_f32, theme.text_secondary);
        visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(6);

        // Inactive widgets (buttons, checkboxes, unselected tabs)
        visuals.widgets.inactive.bg_fill = theme.bg_medium;
        visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0_f32, theme.border);
        visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0_f32, theme.text_primary);
        visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(6);

        // Hovered widgets - smooth tactile feedback
        visuals.widgets.hovered.bg_fill = if is_dark {
            egui::Color32::from_rgb(
                theme.bg_card.r().saturating_add(14),
                theme.bg_card.g().saturating_add(16),
                theme.bg_card.b().saturating_add(22),
            )
        } else {
            egui::Color32::from_rgb(
                theme.bg_card.r().saturating_sub(12),
                theme.bg_card.g().saturating_sub(12),
                theme.bg_card.b().saturating_sub(10),
            )
        };
        visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.2_f32, theme.border_active);
        visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0_f32, theme.text_primary);
        visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(6);

        // Active widgets (pressed, selected)
        visuals.widgets.active.bg_fill = theme.accent_blue.gamma_multiply(0.25);
        visuals.widgets.active.bg_stroke = egui::Stroke::new(1.5_f32, theme.accent_blue);
        visuals.widgets.active.fg_stroke = egui::Stroke::new(1.2_f32, theme.accent_blue);
        visuals.widgets.active.corner_radius = egui::CornerRadius::same(6);

        // Open widgets (e.g. open combo popups)
        visuals.widgets.open.bg_fill = theme.bg_medium;
        visuals.widgets.open.bg_stroke = egui::Stroke::new(1.2_f32, theme.border_active);
        visuals.widgets.open.fg_stroke = egui::Stroke::new(1.0_f32, theme.text_primary);
        visuals.widgets.open.corner_radius = egui::CornerRadius::same(6);

        ctx.set_visuals(visuals);

        let mut style = (*ctx.style()).clone();
        let font_tokens = views::ui_kit::FontTokens::default_tokens();
        font_tokens.apply_to_style(&mut style);
        let sp = views::ui_kit::SpacingTokens::standard();
        style.spacing.item_spacing = egui::vec2(sp.lg, sp.md);
        style.spacing.button_padding = egui::vec2(sp.lg, sp.sm);
        style.spacing.interact_size.y = 36.0;
        style.spacing.text_edit_width = 260.0;
        style.spacing.combo_width = 260.0;
        style.spacing.slider_width = 300.0;
        style.spacing.window_margin = egui::Margin::same(sp.lg as i8);
        ctx.set_style(style);
    }

    fn apply_ui_scale(&self, ctx: &egui::Context) {
        let scale = self
            .state
            .ui
            .ui_scale_percent
            .clamp(MIN_UI_SCALE_PERCENT, MAX_UI_SCALE_PERCENT) as f32
            / 100.0;
        ctx.set_pixels_per_point(scale);
    }

    fn ensure_theme(&mut self, ctx: &egui::Context) {
        if self.applied_dark_mode == Some(self.state.dark_mode) {
            return;
        }
        self.apply_theme(ctx);
        self.applied_dark_mode = Some(self.state.dark_mode);
    }

    fn ensure_ui_scale(&mut self, ctx: &egui::Context) {
        let current = self
            .state
            .ui
            .ui_scale_percent
            .clamp(MIN_UI_SCALE_PERCENT, MAX_UI_SCALE_PERCENT);
        if self.applied_ui_scale_percent == Some(current) {
            return;
        }
        self.apply_ui_scale(ctx);
        self.applied_ui_scale_percent = Some(current);
    }

    fn set_ui_scale(&mut self, percent: u32) {
        let clamped = percent.clamp(MIN_UI_SCALE_PERCENT, MAX_UI_SCALE_PERCENT);
        self.state.ui.ui_scale_percent = clamped;
        self.pending_ui_scale_percent = clamped;
        self.applied_ui_scale_percent = None;
        self.state.status_message = Tr::ui_scale_set(clamped, self.state.lang());
    }

    fn apply_pending_ui_scale(&mut self) {
        self.set_ui_scale(self.pending_ui_scale_percent);
    }

    fn reset_ui_scale(&mut self) {
        self.set_ui_scale(DEFAULT_UI_SCALE_PERCENT);
    }

    fn queue_preferences_save(&mut self, force: bool) {
        let snapshot = match self.state.preferences_snapshot() {
            Ok(snapshot) => snapshot,
            Err(err) => {
                self.state.report_error(err.to_string());
                return;
            }
        };

        if !force
            && self
                .last_saved_prefs_snapshot
                .as_ref()
                .is_some_and(|saved| saved == &snapshot.1)
        {
            return;
        }

        if let Err(err) = self.prefs_save_tx.send((snapshot.0, snapshot.1.clone())) {
            self.state
                .report_error(format!("Preferences save queue failed: {}", err));
            return;
        }

        self.last_saved_prefs_snapshot = Some(snapshot.1);
        self.last_prefs_save = Instant::now();
    }

    fn apply_motion_level_change(&mut self) {
        self.state.apply_performance_profile();
        self.state.resource_status_dirty = true;
        self.state.refresh_resource_status();
    }

    fn tab_telemetry_dot(&self, tab: ActiveTab) -> Option<(egui::Color32, bool)> {
        match tab {
            ActiveTab::Connections => {
                if self.state.active_status().is_connected() || self.state.is_any_connected() {
                    Some((self.state.theme.status_ok, true))
                } else {
                    None
                }
            }
            ActiveTab::SerialDebug => {
                if self.state.active_status().is_connected() || self.state.is_any_connected() {
                    Some((self.state.theme.status_ok, false))
                } else {
                    None
                }
            }
            ActiveTab::PidControl => {
                if self.state.control.is_running {
                    Some((self.state.theme.accent_blue, true))
                } else {
                    None
                }
            }
            ActiveTab::SimulationLab => {
                if self.state.simulation.running {
                    Some((self.state.theme.status_warn, true))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn render_sidebar(&mut self, ctx: &egui::Context, lang: Language) {
        if self.top_tab_mode {
            return;
        }

        let is_expanded = self.state.ui.sidebar_expanded && ctx.screen_rect().width() >= 520.0;
        let sidebar_width = if is_expanded { 190.0 } else { 54.0 };
        self.sidebar_actual_width = sidebar_width;
        let show_expanded_content = is_expanded;
        let theme = self.state.theme.clone();

        egui::SidePanel::left("nav_sidebar")
            .resizable(false)
            .exact_width(sidebar_width)
            .frame(
                egui::Frame::new()
                    .fill(theme.bg_dark)
                    .stroke(egui::Stroke::new(1.0_f32, theme.border)),
            )
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(0.0, 4.0);
                ui.add_space(6.0);

                egui::ScrollArea::vertical()
                    .id_salt("sidebar_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(0.0, 2.0);

                        for group in NAV_GROUPS {
                            if show_expanded_content {
                                ui.add_space(6.0);
                                let group_name = if lang == Language::Chinese {
                                    group.name_zh
                                } else {
                                    group.name_en
                                };
                                ui.horizontal(|ui| {
                                    ui.add_space(14.0);
                                    ui.label(
                                        RichText::new(group_name)
                                            .size(10.5)
                                            .color(theme.text_muted)
                                            .strong(),
                                    );
                                });
                                ui.add_space(2.0);
                            } else {
                                ui.add_space(4.0);
                                let (sep_rect, _) = ui.allocate_exact_size(
                                    egui::vec2(ui.available_width(), 1.0),
                                    egui::Sense::hover(),
                                );
                                let line_rect = egui::Rect::from_min_max(
                                    egui::pos2(sep_rect.left() + 10.0, sep_rect.top()),
                                    egui::pos2(sep_rect.right() - 10.0, sep_rect.bottom()),
                                );
                                ui.painter().rect_filled(
                                    line_rect,
                                    0.5,
                                    theme.border.gamma_multiply(0.6),
                                );
                                ui.add_space(4.0);
                            }

                            for &tab in group.tabs {
                                let selected = self.state.active_tab == tab;
                                let text = tab.label(lang);
                                let icon = tab.icon_kind();
                                let dot = self.tab_telemetry_dot(tab);

                                if show_expanded_content {
                                    let item_size = egui::vec2(ui.available_width() - 8.0, 34.0);
                                    let (rect, response) =
                                        ui.allocate_exact_size(item_size, egui::Sense::click());

                                    let item_rect = egui::Rect::from_min_size(
                                        egui::pos2(rect.left() + 4.0, rect.top()),
                                        egui::vec2(rect.width() - 4.0, rect.height()),
                                    );

                                    let painter = ui.painter();
                                    let hovered = response.hovered();

                                    let (bg_fill, border_stroke, icon_color, text_color) =
                                        if selected {
                                            (
                                                theme.accent_blue.gamma_multiply(0.18),
                                                egui::Stroke::new(
                                                    1.0_f32,
                                                    theme.accent_blue.gamma_multiply(0.40),
                                                ),
                                                theme.accent_blue,
                                                theme.accent_blue,
                                            )
                                        } else if hovered {
                                            (
                                                theme.bg_card,
                                                egui::Stroke::new(1.0_f32, theme.border_active),
                                                theme.text_primary,
                                                theme.text_primary,
                                            )
                                        } else {
                                            (
                                                egui::Color32::TRANSPARENT,
                                                egui::Stroke::NONE,
                                                theme.text_muted,
                                                theme.text_secondary,
                                            )
                                        };

                                    if bg_fill != egui::Color32::TRANSPARENT {
                                        painter.rect_filled(item_rect, 6.0, bg_fill);
                                    }
                                    if border_stroke != egui::Stroke::NONE {
                                        painter.rect_stroke(
                                            item_rect,
                                            6.0,
                                            border_stroke,
                                            egui::StrokeKind::Middle,
                                        );
                                    }

                                    if selected {
                                        let pill = egui::Rect::from_min_max(
                                            egui::pos2(
                                                item_rect.left() + 2.0,
                                                item_rect.top() + 6.0,
                                            ),
                                            egui::pos2(
                                                item_rect.left() + 5.0,
                                                item_rect.bottom() - 6.0,
                                            ),
                                        );
                                        painter.rect_filled(pill, 1.5, theme.accent_blue);
                                    }

                                    let icon_rect = egui::Rect::from_min_size(
                                        egui::pos2(
                                            item_rect.left() + 12.0,
                                            item_rect.center().y - 8.0,
                                        ),
                                        egui::vec2(16.0, 16.0),
                                    );
                                    views::ui_kit::draw_icon(painter, icon_rect, icon, icon_color);

                                    let font_tokens = views::ui_kit::FontTokens::default_tokens();
                                    let text_pos = egui::pos2(
                                        item_rect.left() + 36.0,
                                        item_rect.center().y - 8.0,
                                    );
                                    painter.text(
                                        text_pos,
                                        egui::Align2::LEFT_TOP,
                                        text,
                                        if selected {
                                            font_tokens.button.clone()
                                        } else {
                                            font_tokens.body.clone()
                                        },
                                        text_color,
                                    );

                                    if let Some((dot_color, pulse)) = dot {
                                        let dot_center = egui::pos2(
                                            item_rect.right() - 14.0,
                                            item_rect.center().y,
                                        );
                                        let pulse_alpha = if pulse {
                                            0.65 + 0.35
                                                * (ctx.input(|i| i.time) * 4.0).sin().abs() as f32
                                        } else {
                                            1.0
                                        };
                                        painter.circle_filled(
                                            dot_center,
                                            3.5,
                                            dot_color.gamma_multiply(pulse_alpha),
                                        );
                                        if pulse {
                                            ctx.request_repaint_after(Duration::from_millis(50));
                                        }
                                    }

                                    if response.clicked() {
                                        self.state.active_tab = tab;
                                    }
                                } else {
                                    let item_size = egui::vec2(ui.available_width(), 36.0);
                                    let (rect, response) =
                                        ui.allocate_exact_size(item_size, egui::Sense::click());
                                    let painter = ui.painter();
                                    let hovered = response.hovered();

                                    let icon_box = egui::Rect::from_center_size(
                                        rect.center(),
                                        egui::vec2(36.0, 32.0),
                                    );

                                    let (bg_fill, border_stroke, icon_color) = if selected {
                                        (
                                            theme.accent_blue.gamma_multiply(0.20),
                                            egui::Stroke::new(
                                                1.0_f32,
                                                theme.accent_blue.gamma_multiply(0.40),
                                            ),
                                            theme.accent_blue,
                                        )
                                    } else if hovered {
                                        (
                                            theme.bg_card,
                                            egui::Stroke::new(1.0_f32, theme.border_active),
                                            theme.text_primary,
                                        )
                                    } else {
                                        (
                                            egui::Color32::TRANSPARENT,
                                            egui::Stroke::NONE,
                                            theme.text_muted,
                                        )
                                    };

                                    if bg_fill != egui::Color32::TRANSPARENT {
                                        painter.rect_filled(icon_box, 6.0, bg_fill);
                                    }
                                    if border_stroke != egui::Stroke::NONE {
                                        painter.rect_stroke(
                                            icon_box,
                                            6.0,
                                            border_stroke,
                                            egui::StrokeKind::Middle,
                                        );
                                    }

                                    if selected {
                                        let pill = egui::Rect::from_min_max(
                                            egui::pos2(rect.left() + 2.0, rect.top() + 8.0),
                                            egui::pos2(rect.left() + 5.0, rect.bottom() - 8.0),
                                        );
                                        painter.rect_filled(pill, 1.5, theme.accent_blue);
                                    }

                                    let icon_rect = egui::Rect::from_center_size(
                                        rect.center(),
                                        egui::vec2(18.0, 18.0),
                                    );
                                    views::ui_kit::draw_icon(painter, icon_rect, icon, icon_color);

                                    if let Some((dot_color, pulse)) = dot {
                                        let dot_center = egui::pos2(
                                            icon_box.right() - 5.0,
                                            icon_box.top() + 6.0,
                                        );
                                        let pulse_alpha = if pulse {
                                            0.65 + 0.35
                                                * (ctx.input(|i| i.time) * 4.0).sin().abs() as f32
                                        } else {
                                            1.0
                                        };
                                        painter.circle_filled(
                                            dot_center,
                                            3.0,
                                            dot_color.gamma_multiply(pulse_alpha),
                                        );
                                        if pulse {
                                            ctx.request_repaint_after(Duration::from_millis(50));
                                        }
                                    }

                                    let resp = response.on_hover_text(text);
                                    if resp.clicked() {
                                        self.state.active_tab = tab;
                                    }
                                }
                            }
                        }

                        // Bottom collapse/expand toggle
                        ui.add_space(14.0);
                        if show_expanded_content {
                            let btn_text = if lang == Language::Chinese {
                                "< 折叠侧栏"
                            } else {
                                "< Collapse"
                            };
                            let (rect, resp) = ui.allocate_exact_size(
                                egui::vec2(ui.available_width() - 16.0, 26.0),
                                egui::Sense::click(),
                            );
                            let item_rect = egui::Rect::from_min_size(
                                egui::pos2(rect.left() + 8.0, rect.top()),
                                rect.size(),
                            );
                            let hovered = resp.hovered();
                            if hovered {
                                ui.painter().rect_filled(item_rect, 4.0, theme.bg_card);
                            }
                            let painter = ui.painter();
                            let font_tokens = views::ui_kit::FontTokens::default_tokens();
                            painter.text(
                                item_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                btn_text,
                                font_tokens.caption,
                                if hovered {
                                    theme.text_primary
                                } else {
                                    theme.text_muted
                                },
                            );
                            if resp
                                .on_hover_text(if lang == Language::Chinese {
                                    "折叠侧边栏至图标模式 (Ctrl+B)"
                                } else {
                                    "Collapse to icon rail (Ctrl+B)"
                                })
                                .clicked()
                            {
                                self.state.ui.sidebar_expanded = false;
                            }
                        } else {
                            let (rect, resp) = ui.allocate_exact_size(
                                egui::vec2(ui.available_width(), 28.0),
                                egui::Sense::click(),
                            );
                            let hovered = resp.hovered();
                            if hovered {
                                ui.painter().rect_filled(
                                    egui::Rect::from_center_size(
                                        rect.center(),
                                        egui::vec2(32.0, 24.0),
                                    ),
                                    4.0,
                                    theme.bg_card,
                                );
                            }
                            let painter = ui.painter();
                            painter.text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                ">",
                                egui::FontId::monospace(13.0),
                                if hovered {
                                    theme.text_primary
                                } else {
                                    theme.text_muted
                                },
                            );
                            if resp
                                .on_hover_text(if lang == Language::Chinese {
                                    "展开侧边栏 (Ctrl+B)"
                                } else {
                                    "Expand sidebar (Ctrl+B)"
                                })
                                .clicked()
                            {
                                self.state.ui.sidebar_expanded = true;
                            }
                        }
                        ui.add_space(8.0);
                    });
            });
    }

    fn render_tab_selector(&mut self, ui: &mut egui::Ui, lang: Language, available_width: f32) {
        let theme = self.state.theme.clone();
        let show_tab_strip = available_width >= 600.0;

        if show_tab_strip {
            egui::ScrollArea::horizontal()
                .id_salt("top_tab_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(6.0, 4.0);
                        for &tab in ActiveTab::all() {
                            let selected = self.state.active_tab == tab;
                            let text = tab.label(lang);
                            let icon = tab.icon_kind();

                            let font_tokens = views::ui_kit::FontTokens::default_tokens();
                            let text_width = ui.fonts(|f| {
                                f.layout_no_wrap(
                                    text.to_string(),
                                    font_tokens.button.clone(),
                                    egui::Color32::WHITE,
                                )
                                .size()
                                .x
                            });
                            let item_width = (text_width + 38.0).max(84.0);
                            let item_height = 28.0;

                            let (rect, response) = ui.allocate_exact_size(
                                egui::vec2(item_width, item_height),
                                egui::Sense::click(),
                            );
                            let painter = ui.painter();

                            let (bg_fill, border_stroke, fg_color) = if selected {
                                (
                                    theme.accent_blue.gamma_multiply(0.20),
                                    egui::Stroke::new(1.5_f32, theme.accent_blue),
                                    theme.accent_blue,
                                )
                            } else if response.hovered() {
                                (
                                    theme.bg_medium,
                                    egui::Stroke::new(1.0_f32, theme.border_active),
                                    theme.text_primary,
                                )
                            } else {
                                (
                                    theme.bg_card,
                                    egui::Stroke::new(1.0_f32, theme.border),
                                    theme.text_secondary,
                                )
                            };

                            painter.rect_filled(rect, 6.0, bg_fill);
                            painter.rect_stroke(rect, 6.0, border_stroke, egui::StrokeKind::Middle);

                            if selected {
                                let bar_rect = egui::Rect::from_min_max(
                                    egui::pos2(rect.left() + 8.0, rect.bottom() - 2.5),
                                    egui::pos2(rect.right() - 8.0, rect.bottom()),
                                );
                                painter.rect_filled(bar_rect, 1.5, theme.accent_blue);
                            }

                            let icon_rect = egui::Rect::from_min_size(
                                egui::pos2(rect.left() + 8.0, rect.center().y - 8.0),
                                egui::vec2(16.0, 16.0),
                            );
                            views::ui_kit::draw_icon(painter, icon_rect, icon, fg_color);

                            let text_pos = egui::pos2(rect.left() + 28.0, rect.center().y - 8.0);
                            painter.text(
                                text_pos,
                                egui::Align2::LEFT_TOP,
                                text,
                                font_tokens.button.clone(),
                                fg_color,
                            );

                            if response.clicked() {
                                self.state.active_tab = tab;
                            }
                        }
                    });
                });
        } else {
            ui.horizontal(|ui| {
                let icon = self.state.active_tab.icon_kind();
                let (icon_rect, _) =
                    ui.allocate_exact_size(egui::vec2(20.0, 20.0), egui::Sense::hover());
                views::ui_kit::draw_icon(ui.painter(), icon_rect, icon, theme.accent_blue);

                egui::ComboBox::from_id_salt("top_tab_selector")
                    .width(available_width.clamp(180.0, 320.0))
                    .selected_text(self.state.active_tab.label(lang))
                    .show_ui(ui, |ui| {
                        for &tab in ActiveTab::all() {
                            let icon = tab.icon_kind();
                            ui.horizontal(|ui| {
                                let (r, _) = ui.allocate_exact_size(
                                    egui::vec2(16.0, 16.0),
                                    egui::Sense::hover(),
                                );
                                views::ui_kit::draw_icon(ui.painter(), r, icon, theme.text_muted);
                                ui.selectable_value(
                                    &mut self.state.active_tab,
                                    tab,
                                    tab.label(lang),
                                );
                            });
                        }
                    });
            });
        }
    }

    fn render_active_tab(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        // Continuous non-linear cubic Bézier page transition (ease-out curve)
        const TRANSITION_DURATION: f64 = 0.20; // 200ms
        let (eased, in_transition) = if let Some(start) = self.tab_transition_start {
            let elapsed = ctx.input(|i| i.time) - start;
            if elapsed < TRANSITION_DURATION {
                ctx.request_repaint();
                let t = (elapsed / TRANSITION_DURATION) as f32;
                let curve = app::animation::Easing::Bezier(0.16, 1.0, 0.3, 1.0);
                (curve.evaluate(t), true)
            } else {
                self.tab_transition_start = None;
                (1.0, false)
            }
        } else {
            (1.0, false)
        };

        if in_transition {
            let y_offset = (1.0 - eased) * 12.0;
            ui.add_space(y_offset);
        }

        // Dual-track viewport architecture:
        // Track 1: Fixed-height IDE viewports (fill 100% height, internal scrolling)
        // Track 2: Resilient scrollable document/form viewports (wrapped in outer ScrollArea)
        match self.state.active_tab {
            ActiveTab::SerialDebug => views::serial_debug::show(ui, &mut self.state),
            ActiveTab::ProtocolAnalysis => views::protocol_analysis::show(ui, &mut self.state),
            ActiveTab::DataViz => views::data_viz::show(ui, &mut self.state),
            ActiveTab::SimulationLab => views::simulation_lab::show(ui, &mut self.state),
            ActiveTab::PacketBuilder => views::packet_builder::show(ui, &mut self.state),
            tab => {
                egui::ScrollArea::vertical()
                    .id_salt("document_viewport_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| match tab {
                        ActiveTab::Dashboard => views::dashboard::show(ui, &mut self.state),
                        ActiveTab::Connections => views::connections::show(ui, &mut self.state),
                        ActiveTab::Topology => views::topology::show(ui, &mut self.state),
                        ActiveTab::PidControl => views::pid_control::show(ui, &mut self.state),
                        ActiveTab::NnTuning => views::nn_tuning::show(ui, &mut self.state),
                        ActiveTab::ModbusTools => views::modbus_view::show(ui, &mut self.state),
                        ActiveTab::CanopenTools => views::canopen_view::show(ui, &mut self.state),
                        _ => unreachable!(),
                    });
            }
        }
    }

    fn maybe_auto_save_preferences(&mut self) {
        let interval = Duration::from_secs(self.state.ui.prefs_autosave_interval_sec.max(1) as u64);
        if self.last_prefs_save.elapsed() >= interval {
            self.queue_preferences_save(false);
        }
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let save_shortcut = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::S);
        if ctx.input_mut(|i| i.consume_shortcut(&save_shortcut)) {
            self.queue_preferences_save(true);
            self.state.status_message = Tr::prefs_saved(self.state.lang()).into();
        }

        let clear_shortcut = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::L);
        if ctx.input_mut(|i| i.consume_shortcut(&clear_shortcut)) {
            self.state.log.log_entries.clear();
            self.state.status_message = Tr::logs_cleared(self.state.lang()).into();
        }

        let mut lang_modifiers = egui::Modifiers::COMMAND;
        lang_modifiers.shift = true;
        let language_shortcut = egui::KeyboardShortcut::new(lang_modifiers, egui::Key::L);
        if ctx.input_mut(|i| i.consume_shortcut(&language_shortcut)) {
            self.state.language = self.state.language.toggle();
        }

        let sidebar_shortcut = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::B);
        if ctx.input_mut(|i| i.consume_shortcut(&sidebar_shortcut)) {
            self.state.ui.sidebar_expanded = !self.state.ui.sidebar_expanded;
        }

        if ctx.input(|i| i.key_pressed(egui::Key::F1)) {
            self.show_shortcuts = true;
        }

        if ctx.input(|i| i.key_pressed(egui::Key::F5)) {
            self.state.refresh_ports();
        }

        let zoom_delta = ctx.input(|i| {
            if i.modifiers.ctrl {
                i.raw_scroll_delta.y
            } else {
                0.0
            }
        });
        if zoom_delta.abs() > f32::EPSILON {
            let next = if zoom_delta > 0.0 {
                self.state
                    .ui
                    .ui_scale_percent
                    .saturating_add(UI_SCALE_STEP_PERCENT as u32)
            } else {
                self.state
                    .ui
                    .ui_scale_percent
                    .saturating_sub(UI_SCALE_STEP_PERCENT as u32)
            };
            self.set_ui_scale(next);
        }
    }

    fn render_menu_bar(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let lang = self.state.lang();

        ui.menu_button(Tr::menu_file(lang), |ui| {
            if ui.button(Tr::menu_export_log(lang)).clicked() {
                match self.state.export_logs_csv() {
                    Ok(path) => {
                        self.state.status_message =
                            Tr::logs_exported(&path.display().to_string(), lang);
                    }
                    Err(e) => {
                        self.state
                            .report_error(Tr::logs_export_failed(&e.to_string(), lang));
                    }
                }
                ui.close_menu();
            }

            if ui.button(Tr::menu_preferences(lang)).clicked() {
                self.show_preferences = true;
                ui.close_menu();
            }

            ui.separator();
            if ui.button(Tr::menu_quit(lang)).clicked() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                ui.close_menu();
            }
        });

        ui.menu_button(Tr::menu_edit(lang), |ui| {
            if ui.button(Tr::menu_clear_logs(lang)).clicked() {
                self.state.log.log_entries.clear();
                self.state.status_message = Tr::logs_cleared(lang).into();
                ui.close_menu();
            }

            if ui.button(Tr::menu_copy_frame(lang)).clicked() {
                if let Some(last) = self.state.log.log_entries.back() {
                    let direction = match last.direction {
                        LogDirection::Tx => "TX",
                        LogDirection::Rx => "RX",
                        LogDirection::Info => "INFO",
                    };
                    let line = format!(
                        "[{}][{}][{}] {}",
                        last.timestamp,
                        last.channel,
                        direction,
                        last.format_data()
                    );
                    ctx.copy_text(line);
                    self.state.status_message = Tr::copied_last_frame(lang).into();
                } else {
                    self.state.status_message = Tr::no_logs_to_copy(lang).into();
                }
                ui.close_menu();
            }

            if ui.button(Tr::menu_reset_counters(lang)).clicked() {
                self.state.reset_counters();
                self.state.status_message = Tr::counters_reset_done(lang).into();
                ui.close_menu();
            }
        });

        ui.menu_button(Tr::menu_view(lang), |ui| {
            let sidebar_label = if self.state.ui.sidebar_expanded {
                Tr::menu_hide_sidebar(lang)
            } else {
                Tr::menu_show_sidebar(lang)
            };
            if ui.button(format!("{} (Ctrl+B)", sidebar_label)).clicked() {
                self.state.ui.sidebar_expanded = !self.state.ui.sidebar_expanded;
                ui.close_menu();
            }

            ui.separator();
            ui.horizontal(|ui| {
                ui.label(if lang == Language::Chinese {
                    "导航布局:"
                } else {
                    "Layout:"
                });
                if ui
                    .selectable_label(
                        !self.top_tab_mode,
                        if lang == Language::Chinese {
                            "侧边栏"
                        } else {
                            "Sidebar"
                        },
                    )
                    .clicked()
                {
                    self.top_tab_mode = false;
                    ui.close_menu();
                }
                if ui
                    .selectable_label(
                        self.top_tab_mode,
                        if lang == Language::Chinese {
                            "顶部标签"
                        } else {
                            "Top Tabs"
                        },
                    )
                    .clicked()
                {
                    self.top_tab_mode = true;
                    ui.close_menu();
                }
            });

            ui.separator();
            ui.label(Tr::menu_motion_level(lang));

            let before = self.state.ui.motion_level_idx;
            for idx in 0..=3 {
                ui.selectable_value(
                    &mut self.state.ui.motion_level_idx,
                    idx,
                    Self::motion_level_label(lang, idx),
                );
            }
            if self.state.ui.motion_level_idx != before {
                self.apply_motion_level_change();
                ui.close_menu();
            }

            ui.separator();
            ui.checkbox(&mut self.state.ui.auto_scroll, Tr::auto_scroll(lang));
            ui.horizontal(|ui| {
                ui.label(Tr::display(lang));
                ui.selectable_value(&mut self.state.ui.display_mode, DisplayMode::Hex, "HEX");
                ui.selectable_value(&mut self.state.ui.display_mode, DisplayMode::Ascii, "ASCII");
                ui.selectable_value(&mut self.state.ui.display_mode, DisplayMode::Mixed, "MIXED");
            });

            ui.separator();
            ui.label(format!(
                "{}: {}%",
                if lang == Language::Chinese {
                    "当前生效"
                } else {
                    "Current"
                },
                self.state.ui.ui_scale_percent
            ));
            ui.add(
                egui::Slider::new(
                    &mut self.pending_ui_scale_percent,
                    MIN_UI_SCALE_PERCENT..=MAX_UI_SCALE_PERCENT,
                )
                .text(Tr::menu_ui_scale(lang))
                .suffix("%"),
            );
            ui.horizontal_wrapped(|ui| {
                if ui
                    .button(if lang == Language::Chinese {
                        "应用缩放"
                    } else {
                        "Apply Scale"
                    })
                    .clicked()
                {
                    self.apply_pending_ui_scale();
                    ui.close_menu();
                }
                if ui.button(Tr::menu_ui_scale_reset(lang)).clicked() {
                    self.reset_ui_scale();
                    ui.close_menu();
                }
            });
            if self.pending_ui_scale_percent != self.state.ui.ui_scale_percent {
                ui.small(if lang == Language::Chinese {
                    "拖动滑块只修改待应用值，点击“应用缩放”后才真正生效。"
                } else {
                    "Dragging changes only the pending value. Click Apply Scale to commit it."
                });
            }
            ui.small(if lang == Language::Chinese {
                "快捷缩放：Ctrl + 滚轮"
            } else {
                "Quick zoom: Ctrl + mouse wheel"
            });

            ui.separator();
            let theme_button = if self.state.dark_mode {
                Tr::light_mode(lang)
            } else {
                Tr::dark_mode(lang)
            };
            if ui.button(theme_button).clicked() {
                self.state.theme_transition_start = Some(ctx.input(|i| i.time));
                self.state.dark_mode = !self.state.dark_mode;
                self.state.rebuild_theme();
                self.applied_dark_mode = None;
                ui.close_menu();
            }
        });

        ui.menu_button(Tr::menu_tools(lang), |ui| {
            if ui.button(Tr::menu_mcp_server(lang)).clicked() {
                self.state.toggle_mcp_server();
                ui.close_menu();
            }

            if ui.button(Tr::menu_check_updates(lang)).clicked() {
                let url = self.state.trigger_update_check();
                ctx.open_url(egui::OpenUrl { url, new_tab: true });
                ui.close_menu();
            }
        });

        ui.menu_button(Tr::menu_help(lang), |ui| {
            if ui.button(Tr::menu_about(lang)).clicked() {
                self.show_about = true;
                ui.close_menu();
            }

            if ui.button(Tr::menu_shortcuts(lang)).clicked() {
                self.show_shortcuts = true;
                ui.close_menu();
            }

            if ui.button(Tr::menu_docs(lang)).clicked() {
                let url = self.state.documentation_url();
                ctx.open_url(egui::OpenUrl { url, new_tab: true });
                self.state.status_message = Tr::docs_opened(lang).into();
                ui.close_menu();
            }
        });

        ui.menu_button(Tr::menu_language(lang), |ui| {
            if ui
                .selectable_label(
                    self.state.language == Language::Chinese,
                    Language::Chinese.label(),
                )
                .clicked()
            {
                self.state.language = Language::Chinese;
                ui.close_menu();
            }

            if ui
                .selectable_label(
                    self.state.language == Language::English,
                    Language::English.label(),
                )
                .clicked()
            {
                self.state.language = Language::English;
                ui.close_menu();
            }
        });
    }

    fn render_dialogs(&mut self, ctx: &egui::Context) {
        let lang = self.state.lang();

        if self.show_preferences {
            let mut open = self.show_preferences;
            egui::Window::new(Tr::prefs_title(lang))
                .open(&mut open)
                .resizable(false)
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.set_min_width(420.0);
                    ui.label(Tr::menu_language(lang));
                    ui.horizontal_wrapped(|ui| {
                        ui.selectable_value(
                            &mut self.state.language,
                            Language::Chinese,
                            Language::Chinese.label(),
                        );
                        ui.selectable_value(
                            &mut self.state.language,
                            Language::English,
                            Language::English.label(),
                        );
                    });

                    ui.separator();
                    if ui
                        .checkbox(&mut self.state.dark_mode, Tr::dark_mode(lang))
                        .changed()
                    {
                        self.state.rebuild_theme();
                        self.applied_dark_mode = None;
                    }
                    let hc_label = if lang == Language::Chinese {
                        "高对比度"
                    } else {
                        "High Contrast"
                    };
                    if ui
                        .checkbox(&mut self.state.high_contrast, hc_label)
                        .changed()
                    {
                        self.state.rebuild_theme();
                    }
                    ui.checkbox(&mut self.state.ui.sidebar_expanded, Tr::prefs_sidebar(lang));
                    ui.checkbox(&mut self.state.ui.auto_scroll, Tr::auto_scroll(lang));

                    ui.horizontal_wrapped(|ui| {
                        ui.label(Tr::display(lang));
                        egui::ComboBox::from_id_salt("prefs_display_mode")
                            .width(220.0)
                            .selected_text(match self.state.ui.display_mode {
                                DisplayMode::Hex => "HEX",
                                DisplayMode::Ascii => "ASCII",
                                DisplayMode::Mixed => "MIXED",
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.state.ui.display_mode,
                                    DisplayMode::Hex,
                                    "HEX",
                                );
                                ui.selectable_value(
                                    &mut self.state.ui.display_mode,
                                    DisplayMode::Ascii,
                                    "ASCII",
                                );
                                ui.selectable_value(
                                    &mut self.state.ui.display_mode,
                                    DisplayMode::Mixed,
                                    "MIXED",
                                );
                            });
                    });

                    ui.horizontal_wrapped(|ui| {
                        ui.label(Tr::prefs_motion_level(lang));
                        let before_motion_level = self.state.ui.motion_level_idx;
                        egui::ComboBox::from_id_salt("prefs_motion_level")
                            .width(220.0)
                            .selected_text(Self::motion_level_label(
                                lang,
                                self.state.ui.motion_level_idx,
                            ))
                            .show_ui(ui, |ui| {
                                for idx in 0..=3 {
                                    ui.selectable_value(
                                        &mut self.state.ui.motion_level_idx,
                                        idx,
                                        Self::motion_level_label(lang, idx),
                                    );
                                }
                            });
                        if self.state.ui.motion_level_idx != before_motion_level {
                            self.apply_motion_level_change();
                        }
                    });

                    ui.label(format!(
                        "{}: {}%",
                        if lang == Language::Chinese {
                            "当前生效"
                        } else {
                            "Current"
                        },
                        self.state.ui.ui_scale_percent
                    ));
                    ui.add(
                        egui::Slider::new(
                            &mut self.pending_ui_scale_percent,
                            MIN_UI_SCALE_PERCENT..=MAX_UI_SCALE_PERCENT,
                        )
                        .text(Tr::prefs_ui_scale(lang))
                        .suffix("%"),
                    );
                    ui.horizontal_wrapped(|ui| {
                        if ui
                            .button(if lang == Language::Chinese {
                                "应用缩放"
                            } else {
                                "Apply Scale"
                            })
                            .clicked()
                        {
                            self.apply_pending_ui_scale();
                        }
                        if ui.button(Tr::menu_ui_scale_reset(lang)).clicked() {
                            self.reset_ui_scale();
                        }
                    });
                    ui.small(if lang == Language::Chinese {
                        "快捷缩放：Ctrl + 滚轮"
                    } else {
                        "Quick zoom: Ctrl + mouse wheel"
                    });

                    ui.add(
                        egui::Slider::new(&mut self.state.ui.prefs_autosave_interval_sec, 1..=300)
                            .text(Tr::prefs_autosave_seconds(lang)),
                    );

                    ui.separator();
                    ui.horizontal_wrapped(|ui| {
                        if ui.button(Tr::save(lang)).clicked() {
                            self.queue_preferences_save(true);
                            self.state.status_message = Tr::prefs_saved(lang).into();
                        }
                        if ui.button(Tr::reset(lang)).clicked() {
                            self.state.reset_user_preferences();
                            self.pending_ui_scale_percent = self.state.ui.ui_scale_percent;
                            self.applied_dark_mode = None;
                            self.applied_ui_scale_percent = None;
                            self.apply_motion_level_change();
                        }
                    });
                });
            self.show_preferences = open;
        }

        if self.show_about {
            let mut open = self.show_about;
            egui::Window::new(Tr::menu_about(lang))
                .open(&mut open)
                .resizable(false)
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.set_min_width(360.0);
                    ui.heading(Tr::app_title(lang));
                    ui.label(format!(
                        "{}: {}",
                        if lang == Language::Chinese {
                            "版本"
                        } else {
                            "Version"
                        },
                        self.state.build_version
                    ));
                    ui.label(format!(
                        "{}: {}",
                        Tr::menu_language(lang),
                        self.state.language.label()
                    ));
                    ui.separator();
                    ui.label(Tr::about_summary(lang));
                    if ui.button(Tr::menu_docs(lang)).clicked() {
                        let url = self.state.documentation_url();
                        ctx.open_url(egui::OpenUrl { url, new_tab: true });
                    }
                });
            self.show_about = open;
        }

        if self.show_shortcuts {
            let mut open = self.show_shortcuts;
            egui::Window::new(Tr::shortcuts_title(lang))
                .open(&mut open)
                .resizable(false)
                .collapsible(false)
                .show(ctx, |ui| {
                    let tips = if lang == Language::Chinese {
                        vec![
                            ("Ctrl+B", "展开/折叠侧边栏"),
                            ("Ctrl+S", "保存偏好设置"),
                            ("Ctrl+L", "清除日志"),
                            ("Ctrl+Shift+L", "切换语言"),
                            ("F1", "打开快捷键窗口"),
                            ("F5", "刷新串口"),
                        ]
                    } else {
                        vec![
                            ("Ctrl+B", "Toggle sidebar"),
                            ("Ctrl+S", "Save preferences"),
                            ("Ctrl+L", "Clear logs"),
                            ("Ctrl+Shift+L", "Toggle language"),
                            ("F1", "Open shortcuts"),
                            ("F5", "Refresh serial ports"),
                        ]
                    };

                    egui::Grid::new("shortcuts_grid")
                        .num_columns(2)
                        .spacing([16.0, 8.0])
                        .show(ui, |ui| {
                            for (key, desc) in tips {
                                ui.monospace(key);
                                ui.label(desc);
                                ui.end_row();
                            }
                        });
                });
            self.show_shortcuts = open;
        }
    }
}

impl eframe::App for RobotControlApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.state.active_tab != self.previous_tab {
            self.previous_tab = self.state.active_tab;
            self.tab_transition_start = Some(ctx.input(|i| i.time));
        }

        self.ensure_theme(ctx);
        self.ensure_ui_scale(ctx);
        self.handle_shortcuts(ctx);
        self.state.poll_background_tasks();
        self.state.poll_data();
        self.state.maintain_connection();
        self.maybe_auto_save_preferences();

        let lang = self.state.lang();
        let width = ctx.available_rect().width();
        let is_connected = self.state.active_status().is_connected();
        let theme = self.state.theme.clone();

        egui::TopBottomPanel::top("app_topbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(8.0, 0.0);

                // Quick sidebar toggle button (if in sidebar mode)
                if !self.top_tab_mode {
                    let toggle_tip = if self.state.ui.sidebar_expanded {
                        if lang == Language::Chinese {
                            "折叠侧边栏 (Ctrl+B)"
                        } else {
                            "Collapse Sidebar (Ctrl+B)"
                        }
                    } else {
                        if lang == Language::Chinese {
                            "展开侧边栏 (Ctrl+B)"
                        } else {
                            "Expand Sidebar (Ctrl+B)"
                        }
                    };
                    let (t_rect, t_resp) =
                        ui.allocate_exact_size(egui::vec2(22.0, 22.0), egui::Sense::click());
                    let t_hover = t_resp.hovered();
                    if t_hover {
                        ui.painter().rect_filled(t_rect, 4.0, theme.bg_card);
                    }
                    let icon_sym = if self.state.ui.sidebar_expanded {
                        "<"
                    } else {
                        ">"
                    };
                    ui.painter().text(
                        t_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        icon_sym,
                        egui::FontId::monospace(13.0),
                        if t_hover {
                            theme.text_primary
                        } else {
                            theme.text_muted
                        },
                    );
                    if t_resp.on_hover_text(toggle_tip).clicked() {
                        self.state.ui.sidebar_expanded = !self.state.ui.sidebar_expanded;
                    }
                }

                // Brand Title with vector icon and version badge
                let (icon_r, _) =
                    ui.allocate_exact_size(egui::vec2(18.0, 18.0), egui::Sense::hover());
                views::ui_kit::draw_icon(
                    ui.painter(),
                    icon_r,
                    views::ui_kit::IconKind::Dashboard,
                    theme.accent_blue,
                );
                ui.label(RichText::new(Tr::app_title(lang)).strong().size(14.0));
                let ver = format!("v{}", env!("CARGO_PKG_VERSION"));
                ui.label(RichText::new(ver).size(10.5).color(theme.text_muted));

                ui.separator();

                // Menu bar
                self.render_menu_bar(ui, ctx);

                // Right aligned controls
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(8.0, 0.0);

                    // Connection indicator pill and toggle button
                    if is_connected {
                        if ui
                            .button(RichText::new(Tr::disconnect(lang)).color(theme.status_error))
                            .clicked()
                        {
                            self.state.disconnect_active();
                        }
                        ui.colored_label(theme.status_ok, "●");
                        ui.label(
                            RichText::new(if lang == Language::Chinese {
                                "已连接"
                            } else {
                                "ONLINE"
                            })
                            .size(11.5)
                            .color(theme.status_ok)
                            .strong(),
                        );
                    } else {
                        if ui
                            .button(RichText::new(Tr::connect(lang)).color(theme.accent_blue))
                            .clicked()
                        {
                            if let Err(e) = self.state.connect_active() {
                                self.state.add_info_log(&format!("Connect failed: {e}"));
                            }
                        }
                        ui.colored_label(theme.disconnected_color, "○");
                        ui.label(
                            RichText::new(if lang == Language::Chinese {
                                "未连接"
                            } else {
                                "OFFLINE"
                            })
                            .size(11.5)
                            .color(theme.disconnected_color),
                        );
                    }

                    ui.separator();

                    // Theme Mode Switch
                    let theme_label = if self.state.dark_mode {
                        Tr::light_mode(lang)
                    } else {
                        Tr::dark_mode(lang)
                    };
                    if ui.button(RichText::new(theme_label).size(11.5)).clicked() {
                        self.state.theme_transition_start = Some(ctx.input(|i| i.time));
                        self.state.dark_mode = !self.state.dark_mode;
                        self.state.rebuild_theme();
                        self.applied_dark_mode = None;
                    }
                });
            });

            if self.top_tab_mode {
                ui.add_space(2.0);
                self.render_tab_selector(ui, lang, width);
            }
        });

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(12.0, 4.0);

                // 1. Active Tab with vector icon
                let tab_icon = self.state.active_tab.icon_kind();
                let (icon_r, _) =
                    ui.allocate_exact_size(egui::vec2(14.0, 14.0), egui::Sense::hover());
                views::ui_kit::draw_icon(ui.painter(), icon_r, tab_icon, theme.accent_blue);
                ui.label(
                    RichText::new(self.state.active_tab.label(lang))
                        .strong()
                        .size(12.0)
                        .color(theme.text_primary),
                );

                ui.separator();

                // 2. Link Health with status dot
                let dot_color = if is_connected {
                    theme.status_ok
                } else {
                    theme.disconnected_color
                };
                ui.colored_label(dot_color, "●");
                ui.label(
                    RichText::new(format!(
                        "{}: {}",
                        Tr::top_health(lang),
                        self.state.link_health_text()
                    ))
                    .size(12.0)
                    .color(theme.text_secondary),
                );

                ui.separator();

                // 3. Status message
                let msg = if self.state.status_message.is_empty() {
                    if lang == Language::Chinese {
                        "就绪"
                    } else {
                        "Ready"
                    }
                } else {
                    &self.state.status_message
                };
                ui.label(
                    RichText::new(format!("{}: {}", Tr::top_status(lang), msg))
                        .size(12.0)
                        .color(theme.text_secondary),
                );

                ui.separator();

                // 4. Data Transfer Counters
                ui.label(
                    RichText::new(format!(
                        "TX: {} | RX: {}",
                        views::serial_debug::format_bytes_short(self.state.total_bytes_sent()),
                        views::serial_debug::format_bytes_short(self.state.total_bytes_received()),
                    ))
                    .size(11.5)
                    .color(theme.text_muted)
                    .monospace(),
                );

                ui.separator();

                // 5. Interactive UI Scale pill
                let scale_text = format!(
                    "{}: {}%",
                    Tr::menu_ui_scale(lang),
                    self.state.ui.ui_scale_percent
                );
                let scale_btn =
                    egui::Button::new(RichText::new(scale_text).size(11.5)).frame(false);
                if ui
                    .add(scale_btn)
                    .on_hover_text(if lang == Language::Chinese {
                        "快捷缩放：Ctrl+滚轮 | 点击重置 100%"
                    } else {
                        "Quick zoom: Ctrl+wheel | Click to reset 100%"
                    })
                    .clicked()
                {
                    self.reset_ui_scale();
                }
            });
        });

        self.render_sidebar(ctx, lang);

        let central_response = egui::CentralPanel::default().show(ctx, |ui| {
            self.render_active_tab(ui, ctx);
        });

        // ── Non-linear cubic Bézier tab transition fade overlay ──────────
        if let Some(start) = self.tab_transition_start {
            let elapsed = ctx.input(|i| i.time) - start;
            const TAB_FADE_DURATION: f64 = 0.20;
            if elapsed < TAB_FADE_DURATION {
                let t = (elapsed / TAB_FADE_DURATION) as f32;
                let eased = app::animation::Easing::Bezier(0.16, 1.0, 0.3, 1.0).evaluate(t);
                let alpha = ((1.0 - eased) * 90.0).clamp(0.0, 255.0) as u8;
                if alpha > 0 {
                    let overlay_color = theme.bg_dark.gamma_multiply(alpha as f32 / 255.0);
                    let painter = ctx.layer_painter(egui::LayerId::new(
                        egui::Order::Foreground,
                        egui::Id::new("tab_transition_veil"),
                    ));
                    painter.rect_filled(central_response.response.rect, 0.0, overlay_color);
                }
            }
        }

        // ── Theme transition fade overlay ───────────────────────
        if let Some(start) = self.state.theme_transition_start {
            let now = ctx.input(|i| i.time);
            let elapsed = now - start;
            let duration = 0.3; // 300ms fade
            if elapsed < duration {
                let alpha = (255.0 * (1.0 - elapsed / duration)).clamp(0.0, 255.0) as u8;
                let overlay_color = if self.state.dark_mode {
                    egui::Color32::from_rgba_premultiplied(240, 240, 245, alpha)
                } else {
                    egui::Color32::from_rgba_premultiplied(22, 28, 38, alpha)
                };
                let screen = ctx.screen_rect();
                let painter = ctx.layer_painter(egui::LayerId::new(
                    egui::Order::Foreground,
                    egui::Id::new("theme_transition"),
                ));
                painter.rect_filled(screen, 0.0, overlay_color);
                ctx.request_repaint();
            } else {
                self.state.theme_transition_start = None;
            }
        }

        self.render_dialogs(ctx);

        ctx.request_repaint_after(Duration::from_millis(
            self.effective_repaint_interval_ms(ctx),
        ));
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if self.state.ui.mcp_running {
            self.state.stop_mcp_server();
        }
        self.state.disconnect_active();
        self.state.flush_pending_logs();
        self.state.save_user_preferences();
    }
}

fn maybe_handle_cli_flag() -> bool {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("--version") | Some("-V") => {
            println!("robot_control_rust {}", env!("CARGO_PKG_VERSION"));
            true
        }
        Some("--help") | Some("-h") => {
            println!("Robot Control Suite");
            println!("  --version, -V    Show version");
            println!("  --help, -h       Show help");
            true
        }
        _ => false,
    }
}

/// Try to load a CJK font from the system, with multi-tier fallback.
///
/// Returns `(font_bytes, font_name_hint)` on success.
fn try_load_cjk_font() -> Option<(Vec<u8>, &'static str)> {
    // Tier 1: Modern CJK fonts (best quality, proportional metrics)
    // Tier 2: Legacy CJK fonts (acceptable fallback)
    let candidates: &[(&str, &str)] = if cfg!(target_os = "windows") {
        &[
            ("C:\\Windows\\Fonts\\msyh.ttc", "Microsoft YaHei"),
            ("C:\\Windows\\Fonts\\msyhbd.ttc", "Microsoft YaHei Bold"),
            ("C:\\Windows\\Fonts\\simhei.ttf", "SimHei"),
            ("C:\\Windows\\Fonts\\simsun.ttc", "SimSun"),
        ]
    } else if cfg!(target_os = "macos") {
        &[
            ("/System/Library/Fonts/PingFang.ttc", "PingFang SC"),
            (
                "/System/Library/Fonts/Hiragino Sans GB.ttc",
                "Hiragino Sans GB",
            ),
            ("/System/Library/Fonts/STHeiti Medium.ttc", "STHeiti"),
        ]
    } else {
        &[
            (
                "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
                "WenQuanYi Micro Hei",
            ),
            (
                "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
                "Noto Sans CJK",
            ),
            (
                "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
                "Noto Sans CJK",
            ),
        ]
    };

    for &(path, name) in candidates {
        if let Ok(data) = std::fs::read(path) {
            tracing::info!("CJK font loaded: {} ({})", name, path);
            return Some((data, name));
        }
    }
    tracing::warn!("No CJK font found on system; CJK text may render as placeholders");
    None
}

/// Install CJK font fallback with optimized rendering settings.
fn install_font_fallback(ctx: &egui::Context) {
    let Some((font_data, font_name)) = try_load_cjk_font() else {
        return;
    };

    let mut fonts = egui::FontDefinitions::default();

    // Enable subpixel rendering for CJK glyphs — significantly improves
    // readability at typical UI sizes (12–24 px).
    let font_data_owned = egui::FontData::from_owned(font_data);

    fonts
        .font_data
        .insert("system-cjk".into(), font_data_owned.into());

    // Insert CJK font as highest-priority fallback for both proportional
    // and monospace families so mixed CJK/Latin text renders coherently.
    if let Some(proportional) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
        proportional.insert(0, "system-cjk".into());
    }
    if let Some(monospace) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
        monospace.insert(0, "system-cjk".into());
    }

    ctx.set_fonts(fonts);
    tracing::info!("CJK font fallback installed: {}", font_name);
}

#[cfg(target_os = "linux")]
fn check_linux_env() {
    if std::env::var("WINIT_UNIX_BACKEND").is_err() {
        std::env::set_var("WINIT_UNIX_BACKEND", "wayland,x11");
    }
    if let Ok(groups) = std::process::Command::new("groups").output() {
        let out = String::from_utf8_lossy(&groups.stdout);
        if !out.contains("dialout") && !out.contains("tty") && !out.contains("root") {
            eprintln!("Warning: User is not in dialout or tty group. Serial port access might fail.\nPlease run: sudo usermod -a -G dialout $USER");
        }
    }
}

fn main() -> eframe::Result<()> {
    // 初始化 tokio 运行时（用于 MCP 服务器）
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    let _guard = rt.enter();

    #[cfg(target_os = "linux")]
    check_linux_env();

    if maybe_handle_cli_flag() {
        return Ok(());
    }

    tracing_subscriber::fmt::init();

    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Robot Control Suite")
        .with_inner_size([1600.0, 960.0])
        .with_min_inner_size([1180.0, 760.0]);

    if let Ok(icon) = eframe::icon_data::from_png_bytes(include_bytes!(
        "../../assets/branding/robot_control_app_256.png"
    )) {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Glow,
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Robot Control Suite",
        options,
        Box::new(|cc| {
            install_font_fallback(&cc.egui_ctx);
            Ok(Box::new(RobotControlApp::new()))
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_telemetry_dot_disconnected_by_default() {
        let app = RobotControlApp::new();
        assert_eq!(app.tab_telemetry_dot(ActiveTab::Connections), None);
        assert_eq!(app.tab_telemetry_dot(ActiveTab::SerialDebug), None);
        assert_eq!(app.tab_telemetry_dot(ActiveTab::PidControl), None);
        assert_eq!(app.tab_telemetry_dot(ActiveTab::SimulationLab), None);
    }

    #[test]
    fn test_tab_telemetry_dot_reflects_active_subsystems() {
        let mut app = RobotControlApp::new();

        // 1. Connection active
        app.state.conn.active_conn = crate::models::ConnectionType::Serial;
        app.state.conn.serial.status = crate::models::ConnectionStatus::Connected;
        let conn_dot = app.tab_telemetry_dot(ActiveTab::Connections);
        assert!(conn_dot.is_some());
        let (color, pulse) = conn_dot.unwrap();
        assert_eq!(color, app.state.theme.status_ok);
        assert!(pulse);

        // 2. Control loop running
        app.state.control.is_running = true;
        let pid_dot = app.tab_telemetry_dot(ActiveTab::PidControl);
        assert!(pid_dot.is_some());
        let (color, pulse) = pid_dot.unwrap();
        assert_eq!(color, app.state.theme.accent_blue);
        assert!(pulse);

        // 3. Simulation running
        app.state.simulation.running = true;
        let sim_dot = app.tab_telemetry_dot(ActiveTab::SimulationLab);
        assert!(sim_dot.is_some());
        let (color, pulse) = sim_dot.unwrap();
        assert_eq!(color, app.state.theme.status_warn);
        assert!(pulse);
    }

    #[test]
    fn test_bezier_transition_easing_endpoints() {
        let curve = app::animation::Easing::Bezier(0.16, 1.0, 0.3, 1.0);
        assert!((curve.evaluate(0.0) - 0.0).abs() < 1e-4);
        assert!((curve.evaluate(1.0) - 1.0).abs() < 1e-4);
        assert!(curve.evaluate(0.5) > 0.5); // Ease-out characteristic: faster initial progress
    }
}
