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
use eframe::egui;
use i18n::{Language, Tr};
use std::path::PathBuf;
use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::{Duration, Instant};

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
        let mut visuals = if self.state.dark_mode {
            let mut v = egui::Visuals::dark();
            v.override_text_color = Some(theme.text_primary);
            v
        } else {
            egui::Visuals::light()
        };
        visuals.selection.bg_fill = theme.accent_blue.gamma_multiply(0.75);
        ctx.set_visuals(visuals);

        let mut style = (*ctx.style()).clone();
        let font_tokens = views::ui_kit::FontTokens::default_tokens();
        font_tokens.apply_to_style(&mut style);
        let sp = views::ui_kit::SpacingTokens::standard();
        style.spacing.item_spacing = egui::vec2(sp.lg, sp.lg);
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

    fn render_tab_selector(&mut self, ui: &mut egui::Ui, lang: Language, available_width: f32) {
        let show_tab_strip = self.state.ui.sidebar_expanded && available_width >= 1500.0;

        if show_tab_strip {
            ui.horizontal_wrapped(|ui| {
                for &tab in ActiveTab::all() {
                    let selected = self.state.active_tab == tab;
                    let button = egui::Button::new(tab.label(lang))
                        .selected(selected)
                        .min_size(egui::vec2(132.0, 34.0));
                    if ui.add(button).clicked() {
                        self.state.active_tab = tab;
                    }
                }
            });
        } else {
            egui::ComboBox::from_id_salt("top_tab_selector")
                .width(320.0)
                .selected_text(self.state.active_tab.label(lang))
                .show_ui(ui, |ui| {
                    for &tab in ActiveTab::all() {
                        ui.selectable_value(&mut self.state.active_tab, tab, tab.label(lang));
                    }
                });
        }
    }

    fn render_active_tab(&mut self, ui: &mut egui::Ui) {
        match self.state.active_tab {
            ActiveTab::Dashboard => views::dashboard::show(ui, &mut self.state),
            ActiveTab::Connections => views::connections::show(ui, &mut self.state),
            ActiveTab::SerialDebug => views::serial_debug::show(ui, &mut self.state),
            ActiveTab::ProtocolAnalysis => views::protocol_analysis::show(ui, &mut self.state),
            ActiveTab::PacketBuilder => views::packet_builder::show(ui, &mut self.state),
            ActiveTab::Topology => views::topology::show(ui, &mut self.state),
            ActiveTab::PidControl => views::pid_control::show(ui, &mut self.state),
            ActiveTab::NnTuning => views::nn_tuning::show(ui, &mut self.state),
            ActiveTab::DataViz => views::data_viz::show(ui, &mut self.state),
            ActiveTab::SimulationLab => views::simulation_lab::show(ui, &mut self.state),
            ActiveTab::ModbusTools => views::modbus_view::show(ui, &mut self.state),
            ActiveTab::CanopenTools => views::canopen_view::show(ui, &mut self.state),
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
            if ui.button(sidebar_label).clicked() {
                self.state.ui.sidebar_expanded = !self.state.ui.sidebar_expanded;
                ui.close_menu();
            }

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
                            ("Ctrl+S", "保存偏好设置"),
                            ("Ctrl+L", "清除日志"),
                            ("Ctrl+Shift+L", "切换语言"),
                            ("F1", "打开快捷键窗口"),
                            ("F5", "刷新串口"),
                        ]
                    } else {
                        vec![
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
        self.ensure_theme(ctx);
        self.ensure_ui_scale(ctx);
        self.handle_shortcuts(ctx);
        self.state.poll_background_tasks();
        self.state.poll_data();
        self.state.maintain_connection();
        self.maybe_auto_save_preferences();

        let lang = self.state.lang();
        let width = ctx.available_rect().width();
        let accent = self.state.theme.accent_blue;

        egui::TopBottomPanel::top("app_topbar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                self.render_menu_bar(ui, ctx);
                ui.separator();
                ui.vertical(|ui| {
                    ui.heading(Tr::app_title(lang));
                    ui.colored_label(
                        accent,
                        if lang == Language::Chinese {
                            "连接、诊断、调参与数据分析一体化工作台"
                        } else {
                            "Unified workspace for connection, diagnostics, tuning, and data analysis"
                        },
                    );
                });
                ui.separator();

                if ui
                    .button(if self.state.dark_mode {
                        Tr::light_mode(lang)
                    } else {
                        Tr::dark_mode(lang)
                    })
                    .clicked()
                {
                    self.state.theme_transition_start = Some(ctx.input(|i| i.time));
                    self.state.dark_mode = !self.state.dark_mode;
                    self.state.rebuild_theme();
                    self.applied_dark_mode = None;
                }

                if self.state.active_status().is_connected() {
                    if ui.button(Tr::disconnect(lang)).clicked() {
                        self.state.disconnect_active();
                    }
                } else if ui.button(Tr::connect(lang)).clicked() {
                    if let Err(e) = self.state.connect_active() {
                                    self.state.add_info_log(&format!("Connect failed: {e}"));
                                }
                }
            });
            ui.separator();
            self.render_tab_selector(ui, lang, width);
        });

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(format!(
                    "{}: {}",
                    if lang == Language::Chinese {
                        "当前页面"
                    } else {
                        "Active View"
                    },
                    self.state.active_tab.label(lang)
                ));
                ui.separator();
                ui.label(format!(
                    "{}: {}",
                    Tr::top_health(lang),
                    self.state.link_health_text()
                ));
                ui.separator();
                ui.label(format!(
                    "{}: {}",
                    Tr::top_status(lang),
                    self.state.status_message
                ));
                ui.separator();
                ui.label(format!(
                    "{}: {}%",
                    Tr::menu_ui_scale(lang),
                    self.state.ui.ui_scale_percent
                ));
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_active_tab(ui);
        });

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

    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Glow,
        viewport: egui::ViewportBuilder::default()
            .with_title("Robot Control Suite")
            .with_inner_size([1600.0, 960.0])
            .with_min_inner_size([1180.0, 760.0]),
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
