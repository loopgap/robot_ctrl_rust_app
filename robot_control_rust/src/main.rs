#![windows_subsystem = "windows"]
#![allow(dead_code)]

mod app;
mod i18n;
mod models;
mod services;
mod views;

use app::{ActiveTab, AppState, DisplayMode, LogDirection};
use eframe::egui;
use i18n::{Language, Tr};
use std::time::{Duration, Instant};

struct RobotControlApp {
    state: AppState,
    last_prefs_save: Instant,
    show_preferences: bool,
    show_about: bool,
    show_shortcuts: bool,
}

impl RobotControlApp {
    fn new() -> Self {
        Self {
            state: AppState::new(),
            last_prefs_save: Instant::now(),
            show_preferences: false,
            show_about: false,
            show_shortcuts: false,
        }
    }

    fn repaint_interval_ms(&self) -> u64 {
        match self.state.ui.motion_level_idx {
            0 => 8,
            1 => 16,
            2 => 33,
            _ => 66,
        }
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
        if self.state.dark_mode {
            let mut visuals = egui::Visuals::dark();
            visuals.override_text_color = Some(egui::Color32::from_rgb(220, 220, 230));
            ctx.set_visuals(visuals);
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }
    }

    fn apply_ui_scale(&self, ctx: &egui::Context) {
        let scale = self.state.ui.ui_scale_percent.clamp(80, 160) as f32 / 100.0;
        ctx.set_pixels_per_point(scale);
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
            ActiveTab::ModbusTools => views::modbus_view::show(ui, &mut self.state),
            ActiveTab::CanopenTools => views::canopen_view::show(ui, &mut self.state),
        }
    }

    fn maybe_auto_save_preferences(&mut self) {
        let interval = Duration::from_secs(self.state.ui.prefs_autosave_interval_sec.max(1) as u64);
        if self.last_prefs_save.elapsed() >= interval {
            self.state.save_user_preferences();
            self.last_prefs_save = Instant::now();
        }
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let save_shortcut = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::S);
        if ctx.input_mut(|i| i.consume_shortcut(&save_shortcut)) {
            self.state.save_user_preferences();
            self.state.status_message = Tr::prefs_saved(self.state.lang()).into();
        }

        let clear_shortcut = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::L);
        if ctx.input_mut(|i| i.consume_shortcut(&clear_shortcut)) {
            self.state.log_entries.clear();
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
                        self.state.report_error(Tr::logs_export_failed(&e, lang));
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
                self.state.log_entries.clear();
                self.state.status_message = Tr::logs_cleared(lang).into();
                ui.close_menu();
            }

            if ui.button(Tr::menu_copy_frame(lang)).clicked() {
                if let Some(last) = self.state.log_entries.last() {
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
            let before_scale = self.state.ui.ui_scale_percent;
            ui.add(
                egui::Slider::new(&mut self.state.ui.ui_scale_percent, 80..=160)
                    .text(Tr::menu_ui_scale(lang))
                    .suffix("%"),
            );
            if self.state.ui.ui_scale_percent != before_scale {
                self.state.status_message = Tr::ui_scale_set(self.state.ui.ui_scale_percent, lang);
            }
            if ui.button(Tr::menu_ui_scale_reset(lang)).clicked() {
                self.state.ui.ui_scale_percent = 100;
                self.state.status_message = Tr::ui_scale_set(100, lang);
                ui.close_menu();
            }

            ui.separator();
            let theme_button = if self.state.dark_mode {
                Tr::light_mode(lang)
            } else {
                Tr::dark_mode(lang)
            };
            if ui.button(theme_button).clicked() {
                self.state.dark_mode = !self.state.dark_mode;
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
                let url = self.state.update_doc_url();
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
                    ui.label(Tr::menu_language(lang));
                    ui.horizontal(|ui| {
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
                    ui.checkbox(&mut self.state.dark_mode, Tr::dark_mode(lang));
                    ui.checkbox(&mut self.state.ui.sidebar_expanded, Tr::prefs_sidebar(lang));
                    ui.checkbox(&mut self.state.ui.auto_scroll, Tr::auto_scroll(lang));

                    ui.horizontal(|ui| {
                        ui.label(Tr::display(lang));
                        egui::ComboBox::from_id_salt("prefs_display_mode")
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

                    ui.horizontal(|ui| {
                        ui.label(Tr::prefs_motion_level(lang));
                        egui::ComboBox::from_id_salt("prefs_motion_level")
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
                    });

                    let before_scale = self.state.ui.ui_scale_percent;
                    ui.add(
                        egui::Slider::new(&mut self.state.ui.ui_scale_percent, 80..=160)
                            .text(Tr::prefs_ui_scale(lang))
                            .suffix("%"),
                    );
                    if self.state.ui.ui_scale_percent != before_scale {
                        self.state.status_message =
                            Tr::ui_scale_set(self.state.ui.ui_scale_percent, lang);
                    }

                    ui.add(
                        egui::Slider::new(&mut self.state.ui.prefs_autosave_interval_sec, 1..=300)
                            .text(Tr::prefs_autosave_seconds(lang)),
                    );

                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button(Tr::save(lang)).clicked() {
                            self.state.save_user_preferences();
                            self.state.status_message = Tr::prefs_saved(lang).into();
                        }
                        if ui.button(Tr::reset(lang)).clicked() {
                            self.state.reset_user_preferences();
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
                    ui.heading(Tr::app_title(lang));
                    ui.label(format!("Version: {}", self.state.build_version));
                    ui.separator();
                    ui.label(Tr::about_summary(lang));
                    if ui.button(Tr::menu_docs(lang)).clicked() {
                        let url = self.state.update_doc_url();
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
        self.apply_theme(ctx);
        self.apply_ui_scale(ctx);
        self.handle_shortcuts(ctx);
        self.state.poll_background_tasks();
        self.state.poll_data();
        self.state.maintain_connection();
        self.maybe_auto_save_preferences();

        egui::TopBottomPanel::top("app_topbar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                self.render_menu_bar(ui, ctx);
                let lang = self.state.lang();

                ui.separator();

                if ui
                    .button(if self.state.dark_mode {
                        Tr::light_mode(lang)
                    } else {
                        Tr::dark_mode(lang)
                    })
                    .clicked()
                {
                    self.state.dark_mode = !self.state.dark_mode;
                }

                ui.separator();

                if self.state.ui.sidebar_expanded {
                    for &tab in ActiveTab::all() {
                        let selected = self.state.active_tab == tab;
                        if ui.selectable_label(selected, tab.label(lang)).clicked() {
                            self.state.active_tab = tab;
                        }
                    }
                } else {
                    egui::ComboBox::from_id_salt("top_tab_selector")
                        .selected_text(self.state.active_tab.label(lang))
                        .show_ui(ui, |ui| {
                            for &tab in ActiveTab::all() {
                                ui.selectable_value(
                                    &mut self.state.active_tab,
                                    tab,
                                    tab.label(lang),
                                );
                            }
                        });
                }

                ui.separator();

                if self.state.active_status().is_connected() {
                    if ui.button(Tr::disconnect(lang)).clicked() {
                        self.state.disconnect_active();
                    }
                } else if ui.button(Tr::connect(lang)).clicked() {
                    let _ = self.state.connect_active();
                }

                ui.separator();
                ui.label(format!(
                    "{}: {}",
                    Tr::top_health(lang),
                    self.state.link_health_text()
                ));
                ui.label(format!(
                    "{}: {}",
                    Tr::top_status(lang),
                    self.state.status_message
                ));
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_active_tab(ui);
        });

        self.render_dialogs(ctx);

        ctx.request_repaint_after(Duration::from_millis(self.repaint_interval_ms()));
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

fn try_load_cjk_font() -> Option<Vec<u8>> {
    let candidates: &[&str] = if cfg!(target_os = "windows") {
        &[
            "C:\\Windows\\Fonts\\msyh.ttc",
            "C:\\Windows\\Fonts\\msyh.ttf",
            "C:\\Windows\\Fonts\\simsun.ttc",
            "C:\\Windows\\Fonts\\simhei.ttf",
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

fn install_font_fallback(ctx: &egui::Context) {
    if let Some(font_data) = try_load_cjk_font() {
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
}

fn main() -> eframe::Result<()> {
    if maybe_handle_cli_flag() {
        return Ok(());
    }

    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Robot Control Suite")
            .with_inner_size([1500.0, 900.0])
            .with_min_inner_size([1000.0, 650.0]),
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
