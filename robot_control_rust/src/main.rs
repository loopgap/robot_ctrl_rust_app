#![windows_subsystem = "windows"]
#![allow(dead_code)]

mod app;
mod i18n;
mod models;
mod services;
mod views;

use app::{ActiveTab, AppState};
use eframe::egui;
use std::time::{Duration, Instant};

struct RobotControlApp {
    state: AppState,
    last_prefs_save: Instant,
}

impl RobotControlApp {
    fn new() -> Self {
        Self {
            state: AppState::new(),
            last_prefs_save: Instant::now(),
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
}

impl eframe::App for RobotControlApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.apply_theme(ctx);
        self.state.poll_background_tasks();
        self.state.poll_data();
        self.state.maintain_connection();
        self.maybe_auto_save_preferences();

        let lang = self.state.lang();

        egui::TopBottomPanel::top("app_topbar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                if ui
                    .button(if self.state.dark_mode {
                        "Light"
                    } else {
                        "Dark"
                    })
                    .clicked()
                {
                    self.state.dark_mode = !self.state.dark_mode;
                }

                ui.separator();

                for &tab in ActiveTab::all() {
                    let selected = self.state.active_tab == tab;
                    if ui.selectable_label(selected, tab.label(lang)).clicked() {
                        self.state.active_tab = tab;
                    }
                }

                ui.separator();

                if self.state.active_status().is_connected() {
                    if ui.button("Disconnect").clicked() {
                        self.state.disconnect_active();
                    }
                } else if ui.button("Connect").clicked() {
                    let _ = self.state.connect_active();
                }

                ui.separator();
                ui.label(format!("Health: {}", self.state.link_health_text()));
                ui.label(format!("Status: {}", self.state.status_message));
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_active_tab(ui);
        });

        ctx.request_repaint_after(Duration::from_millis(16));
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
        Box::new(|_cc| Ok(Box::new(RobotControlApp::new()))),
    )
}
