// 在 Windows 上隐藏控制台窗口，仅显示 GUI
#![windows_subsystem = "windows"]
#![allow(dead_code)]

mod app;
mod i18n;
mod models;
mod services;
mod views;

use app::{ActiveTab, AppState};
use eframe::egui;
use i18n::Tr;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tracing::info;

const UI_BUILD_TAG: &str = "UI-20260221-R2";

fn main() -> eframe::Result<()> {
    install_panic_hook();
    env_logger::init();
    let _tracing_guard = init_tracing();
    info!(target: "app", version = env!("CARGO_PKG_VERSION"), "app_start");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1500.0, 900.0])
            .with_min_inner_size([1000.0, 650.0])
            .with_icon(generate_app_icon())
            .with_title(format!(
                "Robot Control & Serial Debug Suite [{}]",
                UI_BUILD_TAG
            )),
        ..Default::default()
    };

    eframe::run_native(
        "Robot Control Suite",
        options,
        Box::new(|cc| {
            // 暗色主题
            let mut visuals = egui::Visuals::dark();
            visuals.override_text_color = Some(egui::Color32::from_rgb(220, 220, 230));
            cc.egui_ctx.set_visuals(visuals);

            // 全局字体
            let mut style = (*cc.egui_ctx.style()).clone();
            style.text_styles.insert(
                egui::TextStyle::Body,
                egui::FontId::new(13.5, egui::FontFamily::Proportional),
            );
            style.text_styles.insert(
                egui::TextStyle::Button,
                egui::FontId::new(13.5, egui::FontFamily::Proportional),
            );
            style.text_styles.insert(
                egui::TextStyle::Heading,
                egui::FontId::new(20.0, egui::FontFamily::Proportional),
            );
            // 增大全局间距
            style.spacing.item_spacing = egui::vec2(8.0, 6.0);
            style.spacing.button_padding = egui::vec2(8.0, 4.0);
            style.spacing.window_margin = egui::Margin::same(12);
            cc.egui_ctx.set_style(style);

            // 加载系统 CJK 字体（中文支持，跨平台）
            let mut fonts = egui::FontDefinitions::default();
            if let Some(cjk_data) = load_cjk_font_data() {
                fonts.font_data.insert(
                    "cjk_font".to_owned(),
                    egui::FontData::from_owned(cjk_data).into(),
                );
                if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                    family.push("cjk_font".to_owned());
                }
                if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
                    family.push("cjk_font".to_owned());
                }
            }
            cc.egui_ctx.set_fonts(fonts);

            Ok(Box::new(RobotApp::new()))
        }),
    )
}

fn init_tracing() -> Option<tracing_appender::non_blocking::WorkerGuard> {
    let dir = app_log_dir();
    if std::fs::create_dir_all(&dir).is_err() {
        return None;
    }
    let file_appender = tracing_appender::rolling::never(dir, "telemetry.log");
    let (writer, guard) = tracing_appender::non_blocking(file_appender);
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        .with_target(true)
        .with_writer(writer)
        .finish();
    if tracing::subscriber::set_global_default(subscriber).is_ok() {
        Some(guard)
    } else {
        None
    }
}

fn app_log_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            return PathBuf::from(appdata)
                .join("robot_control_rust")
                .join("logs");
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("robot_control_rust")
                .join("logs");
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home)
            .join(".config")
            .join("robot_control_rust")
            .join("logs");
    }
    PathBuf::from("logs")
}

fn install_panic_hook() {
    std::panic::set_hook(Box::new(|panic_info| {
        let dir = app_log_dir();
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("panic.log");
        if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(path) {
            let ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
            let _ = writeln!(f, "[{}] panic: {}", ts, panic_info);
            let bt = std::backtrace::Backtrace::capture();
            let _ = writeln!(f, "{}", bt);
            let _ = writeln!(f, "----------------------------------------");
        }
    }));
}

struct RobotApp {
    state: AppState,
    sidebar_width_anim: f32,
    tab_transition_progress: f32,
    last_active_tab: ActiveTab,
    activity_indicator_y: Option<f32>,
    logo_breathe_phase: f32,
    acrylic_sheen_phase: f32,
    icon_select_anim: Vec<f32>,
    icon_hover_anim: Vec<f32>,
    prefs_save_elapsed: f32,
    frame_stall_events: u64,
    worst_frame_ms: f32,
    last_frame_ms: f32,
    motion_autotune_cooldown: f32,
    app_start_time: Instant,
    show_preferences_dialog: bool,
    prefs_custom_path: String,
    prefs_search_query: String,
}

impl RobotApp {
    fn new() -> Self {
        Self {
            state: AppState::new(),
            sidebar_width_anim: SIDEBAR_LABEL_WIDTH,
            tab_transition_progress: 1.0,
            last_active_tab: ActiveTab::Dashboard,
            activity_indicator_y: None,
            logo_breathe_phase: 0.0,
            acrylic_sheen_phase: 0.0,
            icon_select_anim: vec![0.0; ActiveTab::all().len()],
            icon_hover_anim: vec![0.0; ActiveTab::all().len()],
            prefs_save_elapsed: 0.0,
            frame_stall_events: 0,
            worst_frame_ms: 0.0,
            last_frame_ms: 0.0,
            motion_autotune_cooldown: 0.0,
            app_start_time: Instant::now(),
            show_preferences_dialog: false,
            prefs_custom_path: AppState::user_prefs_path().display().to_string(),
            prefs_search_query: String::new(),
        }
    }
}

impl Drop for RobotApp {
    fn drop(&mut self) {
        self.state.flush_pending_logs();
        self.state.save_user_preferences();
        self.state.stop_mcp_server();
    }
}

// ═══════════════════════════════════════════════════════════════
// 主界面布局 - 仿现代 IDE 风格
// ═══════════════════════════════════════════════════════════════

/// 活动条颜色（类似 VS Code 蓝色高亮条）
const ACCENT_COLOR: egui::Color32 = egui::Color32::from_rgb(0, 122, 204);
/// 侧边栏背景色
const SIDEBAR_BG: egui::Color32 = egui::Color32::from_rgb(30, 30, 30);
/// 图标条背景色（比侧边栏更深）
const ICON_BAR_BG: egui::Color32 = egui::Color32::from_rgb(24, 24, 24);
/// 底栏背景色
const STATUS_BAR_BG: egui::Color32 = egui::Color32::from_rgb(0, 122, 204);
/// 图标条宽度
const ICON_BAR_WIDTH: f32 = 52.0;
/// 展开后的侧边栏文字区宽度
const SIDEBAR_LABEL_WIDTH: f32 = 150.0;
#[derive(Clone, Copy)]
enum MotionLevel {
    Ultimate,
    Standard,
    Native,
    Optimized,
}

impl MotionLevel {
    fn from_index(idx: usize) -> Self {
        match idx {
            0 => Self::Ultimate,
            1 => Self::Standard,
            2 => Self::Native,
            _ => Self::Optimized,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Ultimate => "极致",
            Self::Standard => "标准",
            Self::Native => "原生",
            Self::Optimized => "优化",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::Ultimate => "最高流畅与质感，资源占用更高",
            Self::Standard => "流畅优先，适合日常主力使用",
            Self::Native => "接近系统原生节奏，平衡自然",
            Self::Optimized => "低资源占用，过渡更克制",
        }
    }

    fn profile(self) -> MotionProfile {
        match self {
            Self::Ultimate => MotionProfile {
                smoothness: 10,
                texture: 10,
                efficiency: 4,
            },
            Self::Standard => MotionProfile {
                smoothness: 8,
                texture: 8,
                efficiency: 7,
            },
            Self::Native => MotionProfile {
                smoothness: 7,
                texture: 6,
                efficiency: 8,
            },
            Self::Optimized => MotionProfile {
                smoothness: 5,
                texture: 4,
                efficiency: 10,
            },
        }
    }
}

#[derive(Clone, Copy)]
struct MotionProfile {
    smoothness: u8,
    texture: u8,
    efficiency: u8,
}

struct MotionParams {
    sidebar_speed: f32,
    tab_speed: f32,
    indicator_speed: f32,
    tab_slide_px: f32,
    content_slide_px: f32,
    content_lift_px: f32,
    content_fade_power: f32,
    content_sweep_alpha: f32,
    icon_select_speed: f32,
    icon_hover_speed: f32,
    icon_slide_px: f32,
    acrylic_alpha: f32,
    acrylic_sheen_alpha: f32,
    acrylic_sheen_speed: f32,
    logo_speed: f32,
    overlay_alpha: f32,
    active_repaint_ms: u64,
    idle_repaint_ms: u64,
}

struct PlatformMotionTuning {
    speed_scale: f32,
    slide_scale: f32,
    overlay_scale: f32,
}

fn platform_motion_tuning() -> PlatformMotionTuning {
    #[cfg(target_os = "macos")]
    {
        return PlatformMotionTuning {
            speed_scale: 0.90,
            slide_scale: 1.10,
            overlay_scale: 0.92,
        };
    }
    #[cfg(target_os = "windows")]
    {
        return PlatformMotionTuning {
            speed_scale: 1.00,
            slide_scale: 1.00,
            overlay_scale: 1.00,
        };
    }
    #[cfg(target_os = "linux")]
    {
        return PlatformMotionTuning {
            speed_scale: 0.94,
            slide_scale: 0.95,
            overlay_scale: 0.90,
        };
    }
    #[allow(unreachable_code)]
    PlatformMotionTuning {
        speed_scale: 1.0,
        slide_scale: 1.0,
        overlay_scale: 1.0,
    }
}

fn motion_params(level: MotionLevel) -> MotionParams {
    let base = match level {
        MotionLevel::Ultimate => MotionParams {
            sidebar_speed: 12.0,
            tab_speed: 4.8,
            indicator_speed: 12.0,
            tab_slide_px: 8.0,
            content_slide_px: 16.0,
            content_lift_px: 7.0,
            content_fade_power: 1.18,
            content_sweep_alpha: 88.0,
            icon_select_speed: 15.0,
            icon_hover_speed: 18.0,
            icon_slide_px: 5.2,
            acrylic_alpha: 56.0,
            acrylic_sheen_alpha: 44.0,
            acrylic_sheen_speed: 0.62,
            logo_speed: 1.2,
            overlay_alpha: 24.0,
            active_repaint_ms: 12,
            idle_repaint_ms: 34,
        },
        MotionLevel::Standard => MotionParams {
            sidebar_speed: 10.0,
            tab_speed: 4.1,
            indicator_speed: 10.0,
            tab_slide_px: 6.0,
            content_slide_px: 12.0,
            content_lift_px: 5.2,
            content_fade_power: 1.28,
            content_sweep_alpha: 74.0,
            icon_select_speed: 12.5,
            icon_hover_speed: 14.5,
            icon_slide_px: 4.0,
            acrylic_alpha: 48.0,
            acrylic_sheen_alpha: 36.0,
            acrylic_sheen_speed: 0.48,
            logo_speed: 1.0,
            overlay_alpha: 28.0,
            active_repaint_ms: 16,
            idle_repaint_ms: 48,
        },
        MotionLevel::Native => MotionParams {
            sidebar_speed: 8.7,
            tab_speed: 3.5,
            indicator_speed: 8.8,
            tab_slide_px: 4.5,
            content_slide_px: 9.0,
            content_lift_px: 4.0,
            content_fade_power: 1.34,
            content_sweep_alpha: 62.0,
            icon_select_speed: 10.4,
            icon_hover_speed: 12.0,
            icon_slide_px: 2.9,
            acrylic_alpha: 42.0,
            acrylic_sheen_alpha: 28.0,
            acrylic_sheen_speed: 0.38,
            logo_speed: 0.9,
            overlay_alpha: 32.0,
            active_repaint_ms: 16,
            idle_repaint_ms: 70,
        },
        MotionLevel::Optimized => MotionParams {
            sidebar_speed: 6.8,
            tab_speed: 2.8,
            indicator_speed: 7.0,
            tab_slide_px: 3.0,
            content_slide_px: 6.0,
            content_lift_px: 2.6,
            content_fade_power: 1.46,
            content_sweep_alpha: 48.0,
            icon_select_speed: 8.2,
            icon_hover_speed: 9.2,
            icon_slide_px: 2.1,
            acrylic_alpha: 34.0,
            acrylic_sheen_alpha: 20.0,
            acrylic_sheen_speed: 0.26,
            logo_speed: 0.7,
            overlay_alpha: 22.0,
            active_repaint_ms: 24,
            idle_repaint_ms: 120,
        },
    };

    let tuning = platform_motion_tuning();
    MotionParams {
        sidebar_speed: base.sidebar_speed * tuning.speed_scale,
        tab_speed: base.tab_speed * tuning.speed_scale,
        indicator_speed: base.indicator_speed * tuning.speed_scale,
        tab_slide_px: base.tab_slide_px * tuning.slide_scale,
        content_slide_px: base.content_slide_px * tuning.slide_scale,
        content_lift_px: base.content_lift_px * tuning.slide_scale,
        content_fade_power: base.content_fade_power,
        content_sweep_alpha: base.content_sweep_alpha * tuning.overlay_scale,
        icon_select_speed: base.icon_select_speed * tuning.speed_scale,
        icon_hover_speed: base.icon_hover_speed * tuning.speed_scale,
        icon_slide_px: base.icon_slide_px * tuning.slide_scale,
        acrylic_alpha: base.acrylic_alpha * tuning.overlay_scale,
        acrylic_sheen_alpha: base.acrylic_sheen_alpha * tuning.overlay_scale,
        acrylic_sheen_speed: base.acrylic_sheen_speed,
        logo_speed: base.logo_speed,
        overlay_alpha: base.overlay_alpha * tuning.overlay_scale,
        active_repaint_ms: base.active_repaint_ms,
        idle_repaint_ms: base.idle_repaint_ms,
    }
}

impl eframe::App for RobotApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let tab_count = ActiveTab::all().len();
        if self.icon_select_anim.len() != tab_count {
            self.icon_select_anim.resize(tab_count, 0.0);
        }
        if self.icon_hover_anim.len() != tab_count {
            self.icon_hover_anim.resize(tab_count, 0.0);
        }

        let dt = ctx.input(|i| i.stable_dt).max(0.0001);
        let dt_ms = dt * 1000.0;
        self.prefs_save_elapsed += dt;
        self.last_frame_ms = dt_ms;
        self.worst_frame_ms = self.worst_frame_ms.max(dt_ms);

        if dt_ms > 250.0 {
            self.frame_stall_events = self.frame_stall_events.saturating_add(1);
            if self.frame_stall_events <= 3 || self.frame_stall_events.is_multiple_of(10) {
                self.state.add_info_log(&format!(
                    "UI stall detected: {:.0} ms (count={})",
                    dt_ms, self.frame_stall_events
                ));
            }
        }

        if self.motion_autotune_cooldown > 0.0 {
            self.motion_autotune_cooldown = (self.motion_autotune_cooldown - dt).max(0.0);
        }

        if dt_ms > 220.0
            && self.motion_autotune_cooldown <= 0.0
            && self.state.ui.motion_level_idx < 3
        {
            self.state.ui.motion_level_idx += 1;
            self.motion_autotune_cooldown = 10.0;
            let level = MotionLevel::from_index(self.state.ui.motion_level_idx);
            self.state.add_info_log(&format!(
                "Auto performance tune: switched motion level to {}",
                level.label()
            ));
            self.state.status_message =
                format!("Auto tuned motion to {} (high frame time)", level.label());
        }

        if !ctx.wants_keyboard_input() {
            let shortcut = ctx.input(|i| {
                (
                    i.modifiers.ctrl,
                    i.key_pressed(egui::Key::Num1),
                    i.key_pressed(egui::Key::Num2),
                    i.key_pressed(egui::Key::Num3),
                    i.key_pressed(egui::Key::Num4),
                    i.key_pressed(egui::Key::Num5),
                    i.key_pressed(egui::Key::Num6),
                    i.key_pressed(egui::Key::Num7),
                    i.key_pressed(egui::Key::Num8),
                    i.key_pressed(egui::Key::Num9),
                    i.key_pressed(egui::Key::B),
                    i.key_pressed(egui::Key::M),
                )
            });

            if shortcut.0 {
                let mut goto = None;
                if shortcut.1 {
                    goto = Some(0)
                } else if shortcut.2 {
                    goto = Some(1)
                } else if shortcut.3 {
                    goto = Some(2)
                } else if shortcut.4 {
                    goto = Some(3)
                } else if shortcut.5 {
                    goto = Some(4)
                } else if shortcut.6 {
                    goto = Some(5)
                } else if shortcut.7 {
                    goto = Some(6)
                } else if shortcut.8 {
                    goto = Some(7)
                } else if shortcut.9 {
                    goto = Some(8)
                }
                if let Some(idx) = goto {
                    if let Some(tab) = ActiveTab::all().get(idx) {
                        self.state.active_tab = *tab;
                    }
                }
                if shortcut.10 {
                    self.state.ui.sidebar_expanded = !self.state.ui.sidebar_expanded;
                }
                if shortcut.11 {
                    self.state.ui.motion_level_idx = (self.state.ui.motion_level_idx + 1) % 4;
                }
            }
        }
        let motion_level = MotionLevel::from_index(self.state.ui.motion_level_idx);
        let motion = motion_params(motion_level);

        let icon_hover_peak = self.icon_hover_anim.iter().copied().fold(0.0_f32, f32::max);
        let icon_select_peak = self
            .icon_select_anim
            .iter()
            .copied()
            .fold(0.0_f32, f32::max);
        let acrylic_energy = (icon_select_peak * 0.82 + icon_hover_peak * 0.58).clamp(0.0, 1.0);

        self.logo_breathe_phase =
            (self.logo_breathe_phase + dt * motion.logo_speed) % std::f32::consts::TAU;
        if acrylic_energy > 0.02 {
            let sheen_speed = motion.acrylic_sheen_speed * (0.35 + acrylic_energy * 0.95);
            self.acrylic_sheen_phase = (self.acrylic_sheen_phase + dt * sheen_speed).fract();
        }

        if self.last_active_tab != self.state.active_tab {
            self.last_active_tab = self.state.active_tab;
            self.tab_transition_progress = 0.0;
        }
        self.tab_transition_progress =
            (self.tab_transition_progress + dt * motion.tab_speed).min(1.0);
        let tab_transition_eased = ease_apple_out(self.tab_transition_progress);
        let tab_transition_soft = ease_apple_standard(self.tab_transition_progress);
        let tab_slide = (1.0 - tab_transition_soft) * motion.tab_slide_px;
        let content_slide = (1.0 - tab_transition_eased) * motion.content_slide_px;
        let content_lift = (1.0 - tab_transition_soft) * motion.content_lift_px;

        let sidebar_target = if self.state.ui.sidebar_expanded {
            SIDEBAR_LABEL_WIDTH
        } else {
            0.0
        };
        self.sidebar_width_anim = exp_smooth(
            self.sidebar_width_anim,
            sidebar_target,
            dt,
            motion.sidebar_speed,
        );
        let sidebar_visibility = (self.sidebar_width_anim / SIDEBAR_LABEL_WIDTH).clamp(0.0, 1.0);

        self.state.poll_data();
        self.state.poll_background_tasks();
        self.state.maintain_connection();

        let autosave_interval = self.state.ui.prefs_autosave_interval_sec.clamp(1, 300) as f32;
        if self.prefs_save_elapsed >= autosave_interval {
            self.state.save_user_preferences();
            self.prefs_save_elapsed = 0.0;
        }

        let tab_animating = self.tab_transition_progress < 0.995;
        let sidebar_animating = (self.sidebar_width_anim - sidebar_target).abs() > 0.35;
        let icon_animating = self
            .icon_select_anim
            .iter()
            .chain(self.icon_hover_anim.iter())
            .any(|v| *v > 0.005 && *v < 0.995);
        let animation_active =
            tab_animating || sidebar_animating || icon_animating || acrylic_energy > 0.04;

        let repaint_ms =
            if self.state.is_running || self.state.is_any_connected() || animation_active {
                motion.active_repaint_ms
            } else {
                motion.idle_repaint_ms
            };
        ctx.request_repaint_after(Duration::from_millis(repaint_ms));

        let lang = self.state.language;

        // ─── 底部状态栏 (VS Code 风格蓝色条) ─────────────
        egui::TopBottomPanel::bottom("status_bar")
            .exact_height(28.0)
            .frame(
                egui::Frame::new()
                    .fill(if self.state.dark_mode {
                        STATUS_BAR_BG
                    } else {
                        egui::Color32::from_rgb(0, 100, 180)
                    })
                    .inner_margin(egui::Margin::symmetric(12, 4)),
            )
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    // 左侧：连接状态
                    let status = self.state.active_status();
                    let (r, g, b) = status.color_rgb();
                    ui.label(
                        egui::RichText::new(format!("Status: {}", status))
                            .size(11.5)
                            .color(egui::Color32::from_rgb(r, g, b)),
                    );
                    ui.separator();
                    ui.label(
                        egui::RichText::new(&self.state.status_message)
                            .size(11.5)
                            .color(egui::Color32::WHITE),
                    );
                    ui.separator();
                    ui.label(
                        egui::RichText::new(format!("Link: {}", self.state.link_health_text()))
                            .size(11.5)
                            .color(egui::Color32::from_rgb(225, 235, 255)),
                    );

                    // 右侧：统计与时间
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let frame_color = if self.last_frame_ms < 25.0 {
                            egui::Color32::from_rgb(180, 240, 180)
                        } else if self.last_frame_ms < 80.0 {
                            egui::Color32::from_rgb(240, 230, 170)
                        } else {
                            egui::Color32::from_rgb(255, 180, 160)
                        };
                        ui.label(
                            egui::RichText::new(format!(
                                "Frame:{:.0}ms Spike:{}",
                                self.last_frame_ms, self.frame_stall_events
                            ))
                            .size(11.5)
                            .color(frame_color),
                        );
                        ui.separator();
                        if self.app_start_time.elapsed().as_secs() < 15 {
                            ui.label(
                                egui::RichText::new("Startup optimizer active")
                                    .size(11.5)
                                    .color(egui::Color32::from_rgb(220, 220, 255)),
                            );
                            ui.separator();
                        }
                        ui.label(
                            egui::RichText::new(
                                chrono::Local::now().format("%H:%M:%S").to_string(),
                            )
                            .size(11.5)
                            .color(egui::Color32::from_rgb(220, 220, 220)),
                        );
                        ui.separator();
                        ui.label(
                            egui::RichText::new(format!(
                                "TX:{} RX:{}",
                                format_bytes_compact(self.state.total_bytes_sent()),
                                format_bytes_compact(self.state.total_bytes_received()),
                            ))
                            .size(11.5)
                            .color(egui::Color32::from_rgb(220, 220, 220)),
                        );
                        ui.separator();
                        ui.label(
                            egui::RichText::new(format!("Err:{}", self.state.last_error_time))
                                .size(11.5)
                                .color(egui::Color32::from_rgb(255, 220, 180)),
                        );
                        ui.separator();
                        ui.label(
                            egui::RichText::new(format!("v{}", self.state.build_version))
                                .size(11.5)
                                .color(egui::Color32::from_rgb(220, 220, 220)),
                        );
                        ui.separator();
                        ui.label(
                            egui::RichText::new(UI_BUILD_TAG)
                                .size(11.5)
                                .color(egui::Color32::from_rgb(220, 220, 220)),
                        );
                        ui.separator();
                        // 语言切换按钮
                        let lang_btn = egui::RichText::new(format!("Lang {}", lang.label()))
                            .size(11.5)
                            .color(egui::Color32::WHITE);
                        if ui.button(lang_btn).clicked() {
                            self.state.language = self.state.language.toggle();
                        }

                        ui.separator();
                        let combo_resp = egui::ComboBox::from_id_salt("motion_level_combo")
                            .selected_text(format!("动效: {}", motion_level.label()))
                            .width(108.0)
                            .show_ui(ui, |ui| {
                                for idx in 0..=3 {
                                    let level = MotionLevel::from_index(idx);
                                    let selected = self.state.ui.motion_level_idx == idx;
                                    let resp = ui.selectable_label(
                                        selected,
                                        format!("动效: {}", level.label()),
                                    );
                                    if resp.clicked() {
                                        self.state.ui.motion_level_idx = idx;
                                    }
                                    resp.on_hover_text(level.description());
                                }
                            });
                        combo_resp.response.on_hover_text(format!(
                            "{}\n{}",
                            motion_level.label(),
                            motion_level.description()
                        ));

                        ui.separator();
                        draw_motion_profile_chip(ui, motion_level, self.state.dark_mode);
                    });
                });
            });

        // ─── 顶部企业级菜单栏 (Enterprise Menu Bar) ──────
        egui::TopBottomPanel::top("menu_bar")
            .exact_height(28.0)
            .frame(
                egui::Frame::new()
                    .fill(if self.state.dark_mode {
                        egui::Color32::from_rgb(32, 32, 32)
                    } else {
                        egui::Color32::from_rgb(246, 246, 246)
                    })
                    .inner_margin(egui::Margin::symmetric(6, 0))
                    .stroke(egui::Stroke::new(
                        0.5,
                        if self.state.dark_mode {
                            egui::Color32::from_rgb(50, 50, 55)
                        } else {
                            egui::Color32::from_rgb(210, 210, 210)
                        },
                    )),
            )
            .show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    let menu_text_color = if self.state.dark_mode {
                        egui::Color32::from_rgb(210, 210, 220)
                    } else {
                        egui::Color32::from_rgb(40, 40, 50)
                    };

                    // ─── File ─────────────────────────
                    ui.menu_button(
                        egui::RichText::new(Tr::menu_file(lang)).size(12.5).color(menu_text_color),
                        |ui| {
                            if ui.button(Tr::menu_export_log(lang)).clicked() {
                                if let Ok(path) = crate::views::protocol_analysis::export_analysis_csv(&self.state) {
                                    self.state.status_message = format!("Exported: {}", path.display());
                                }
                                ui.close_menu();
                            }
                            if ui.button(Tr::menu_import_preset(lang)).clicked() {
                                self.state.status_message = "Import preset...".into();
                                ui.close_menu();
                            }
                            ui.separator();
                            if ui.button(Tr::menu_preferences(lang)).clicked() {
                                self.show_preferences_dialog = true;
                                self.prefs_custom_path = AppState::user_prefs_path().display().to_string();
                                self.prefs_search_query.clear();
                                self.state.status_message = "Preferences opened".into();
                                ui.close_menu();
                            }
                            ui.separator();
                            if ui.button(Tr::menu_quit(lang)).clicked() {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                        },
                    );

                    // ─── Edit ─────────────────────────
                    ui.menu_button(
                        egui::RichText::new(Tr::menu_edit(lang)).size(12.5).color(menu_text_color),
                        |ui| {
                            if ui.button(Tr::menu_clear_logs(lang)).clicked() {
                                self.state.log_entries.clear();
                                self.state.status_message = "Logs cleared".into();
                                ui.close_menu();
                            }
                            if ui.button(Tr::menu_copy_frame(lang)).clicked() {
                                if let Some(last) = self.state.log_entries.last() {
                                    let hex: String = last.data.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
                                    ctx.copy_text(hex);
                                    self.state.status_message = "Frame copied".into();
                                }
                                ui.close_menu();
                            }
                            ui.separator();
                            if ui.button(Tr::menu_reset_counters(lang)).clicked() {
                                self.state.reset_counters();
                                self.state.status_message = "Counters reset".into();
                                ui.close_menu();
                            }
                        },
                    );

                    // ─── View ─────────────────────────
                    ui.menu_button(
                        egui::RichText::new(Tr::menu_view(lang)).size(12.5).color(menu_text_color),
                        |ui| {
                            let dark_label = if self.state.dark_mode { "☀ Light Theme" } else { "🌙 Dark Theme" };
                            if ui.button(dark_label).clicked() {
                                self.state.dark_mode = !self.state.dark_mode;
                                if self.state.dark_mode {
                                    let mut v = egui::Visuals::dark();
                                    v.override_text_color = Some(egui::Color32::from_rgb(220, 220, 230));
                                    ctx.set_visuals(v);
                                } else {
                                    ctx.set_visuals(egui::Visuals::light());
                                }
                                ui.close_menu();
                            }
                            let sidebar_label = if self.state.ui.sidebar_expanded { Tr::menu_hide_sidebar(lang) } else { Tr::menu_show_sidebar(lang) };
                            if ui.button(sidebar_label).clicked() {
                                self.state.ui.sidebar_expanded = !self.state.ui.sidebar_expanded;
                                ui.close_menu();
                            }
                            ui.separator();
                            ui.menu_button(Tr::menu_motion_level(lang), |ui| {
                                for idx in 0..=3 {
                                    let level = MotionLevel::from_index(idx);
                                    let selected = self.state.ui.motion_level_idx == idx;
                                    let label = if selected { format!("✓ {}", level.label()) } else { level.label().to_string() };
                                    if ui.button(label).clicked() {
                                        self.state.ui.motion_level_idx = idx;
                                        ui.close_menu();
                                    }
                                }
                            });
                            ui.separator();
                            ui.menu_button(Tr::menu_language(lang), |ui| {
                                let langs = [("English", i18n::Language::English), ("中文", i18n::Language::Chinese)];
                                for (label, l) in &langs {
                                    let selected = self.state.language == *l;
                                    let text = if selected { format!("✓ {}", label) } else { label.to_string() };
                                    if ui.button(text).clicked() {
                                        self.state.language = *l;
                                        ui.close_menu();
                                    }
                                }
                            });
                        },
                    );

                    // ─── Tools ────────────────────────
                    ui.menu_button(
                        egui::RichText::new(Tr::menu_tools(lang)).size(12.5).color(menu_text_color),
                        |ui| {
                            for &tab in ActiveTab::all() {
                                if ui.button(tab.label(lang)).clicked() {
                                    self.state.active_tab = tab;
                                    ui.close_menu();
                                }
                            }
                            ui.separator();
                            if ui.button(Tr::menu_mcp_server(lang)).clicked() {
                                self.state.toggle_mcp_server();
                                ui.close_menu();
                            }
                        },
                    );

                    // ─── Help ─────────────────────────
                    ui.menu_button(
                        egui::RichText::new(Tr::menu_help(lang)).size(12.5).color(menu_text_color),
                        |ui| {
                            if ui.button(Tr::menu_about(lang)).clicked() {
                                self.state.status_message = format!(
                                    "Robot Control Suite v{} | {} | egui 0.31",
                                    self.state.build_version, UI_BUILD_TAG
                                );
                                ui.close_menu();
                            }
                            if ui.button(Tr::menu_shortcuts(lang)).clicked() {
                                self.state.status_message =
                                    "Ctrl+1..9: Switch tab | Ctrl+B: Toggle sidebar | Ctrl+M: Motion level".into();
                                ui.close_menu();
                            }
                            if ui.button(Tr::menu_docs(lang)).clicked() {
                                self.state.status_message = "See ARCHITECTURE_AND_USAGE.md for documentation".into();
                                ui.close_menu();
                            }
                        },
                    );
                });
            });

        if self.show_preferences_dialog {
            let mut open = self.show_preferences_dialog;
            egui::Window::new(if lang == i18n::Language::Chinese {
                "企业级偏好设置"
            } else {
                "Enterprise Preferences"
            })
            .open(&mut open)
            .default_width(760.0)
            .default_height(640.0)
            .resizable(true)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(if lang == i18n::Language::Chinese {
                            "搜索设置"
                        } else {
                            "Search"
                        });
                        ui.text_edit_singleline(&mut self.prefs_search_query);
                        if ui
                            .button(if lang == i18n::Language::Chinese {
                                "清空"
                            } else {
                                "Clear"
                            })
                            .clicked()
                        {
                            self.prefs_search_query.clear();
                        }
                    });
                    let prefs_query = self.prefs_search_query.trim().to_lowercase();
                    let show_group = |tags: &[&str]| {
                        prefs_query.is_empty()
                            || tags.iter().any(|tag| {
                                let low = tag.to_lowercase();
                                low.contains(&prefs_query) || prefs_query.contains(&low)
                            })
                    };

                    ui.label(if lang == i18n::Language::Chinese {
                        "配置路径（支持导入/导出）"
                    } else {
                        "Preferences path (for import/export)"
                    });
                    ui.text_edit_singleline(&mut self.prefs_custom_path);

                    ui.horizontal_wrapped(|ui| {
                        if ui
                            .button(if lang == i18n::Language::Chinese {
                                "保存到默认路径"
                            } else {
                                "Save default"
                            })
                            .clicked()
                        {
                            self.state.save_user_preferences();
                            self.state.status_message = "Preferences saved".into();
                        }

                        if ui
                            .button(if lang == i18n::Language::Chinese {
                                "从默认路径加载"
                            } else {
                                "Load default"
                            })
                            .clicked()
                        {
                            self.state.load_user_preferences();
                        }

                        if ui
                            .button(if lang == i18n::Language::Chinese {
                                "保存到当前路径"
                            } else {
                                "Save to path"
                            })
                            .clicked()
                        {
                            self.state.save_user_preferences_as(std::path::Path::new(
                                &self.prefs_custom_path,
                            ));
                        }

                        if ui
                            .button(if lang == i18n::Language::Chinese {
                                "从当前路径加载"
                            } else {
                                "Load from path"
                            })
                            .clicked()
                        {
                            self.state.load_user_preferences_from(std::path::Path::new(
                                &self.prefs_custom_path,
                            ));
                        }

                        if ui
                            .button(if lang == i18n::Language::Chinese {
                                "恢复默认"
                            } else {
                                "Reset defaults"
                            })
                            .clicked()
                        {
                            self.state.reset_user_preferences();
                        }

                        if ui
                            .button(if lang == i18n::Language::Chinese {
                                "运行系统检查"
                            } else {
                                "Run system check"
                            })
                            .clicked()
                        {
                            self.state.run_system_check();
                        }
                    });

                    ui.separator();

                    if show_group(&[
                        "ui",
                        "theme",
                        "sidebar",
                        "motion",
                        "autosave",
                        "update",
                        "version",
                        "界面",
                        "主题",
                        "侧栏",
                        "动效",
                        "自动保存",
                        "更新",
                        "版本",
                    ]) {
                        ui.collapsing(
                            if lang == i18n::Language::Chinese {
                                "界面与体验"
                            } else {
                                "UI & Experience"
                            },
                            |ui| {
                            if ui
                                .checkbox(
                                    &mut self.state.dark_mode,
                                    if lang == i18n::Language::Chinese {
                                        "暗色主题"
                                    } else {
                                        "Dark mode"
                                    },
                                )
                                .changed()
                            {
                                if self.state.dark_mode {
                                    let mut v = egui::Visuals::dark();
                                    v.override_text_color =
                                        Some(egui::Color32::from_rgb(220, 220, 230));
                                    ctx.set_visuals(v);
                                } else {
                                    ctx.set_visuals(egui::Visuals::light());
                                }
                            }
                            ui.checkbox(
                                &mut self.state.ui.sidebar_expanded,
                                if lang == i18n::Language::Chinese {
                                    "展开侧栏"
                                } else {
                                    "Sidebar expanded"
                                },
                            );

                            ui.horizontal(|ui| {
                                ui.label(if lang == i18n::Language::Chinese {
                                    "动效档位"
                                } else {
                                    "Motion level"
                                });
                                for idx in 0..=3 {
                                    let level = MotionLevel::from_index(idx);
                                    ui.selectable_value(
                                        &mut self.state.ui.motion_level_idx,
                                        idx,
                                        level.label(),
                                    );
                                }
                            });

                            ui.horizontal(|ui| {
                                ui.label(if lang == i18n::Language::Chinese {
                                    "偏好自动保存（秒）"
                                } else {
                                    "Autosave interval (sec)"
                                });
                                ui.add(
                                    egui::DragValue::new(
                                        &mut self.state.ui.prefs_autosave_interval_sec,
                                    )
                                    .range(1..=300)
                                    .speed(1),
                                );
                            });

                            ui.separator();
                            ui.label(if lang == i18n::Language::Chinese {
                                "版本与更新（0.1.x）"
                            } else {
                                "Version & Updates (0.1.x)"
                            });
                            ui.horizontal(|ui| {
                                ui.label(if lang == i18n::Language::Chinese {
                                    "更新通道"
                                } else {
                                    "Update channel"
                                });
                                egui::ComboBox::from_id_salt("prefs_update_channel")
                                    .selected_text(&self.state.ui.update_channel)
                                    .show_ui(ui, |ui| {
                                        for channel in ["stable-0.1", "preview-0.1", "all"] {
                                            ui.selectable_value(
                                                &mut self.state.ui.update_channel,
                                                channel.to_string(),
                                                channel,
                                            );
                                        }
                                    });
                            });
                            ui.horizontal(|ui| {
                                ui.label(if lang == i18n::Language::Chinese {
                                    "更新清单 URL"
                                } else {
                                    "Manifest URL"
                                });
                                ui.text_edit_singleline(&mut self.state.ui.update_manifest_url);
                            });
                            ui.horizontal(|ui| {
                                ui.label(if lang == i18n::Language::Chinese {
                                    "检查超时(ms)"
                                } else {
                                    "Check timeout(ms)"
                                });
                                ui.add(
                                    egui::DragValue::new(&mut self.state.ui.update_check_timeout_ms)
                                        .range(500..=10000)
                                        .speed(50),
                                );
                            });
                            },
                        );
                    }

                    if show_group(&[
                        "terminal",
                        "tx",
                        "display",
                        "newline",
                        "repeat",
                        "终端",
                        "发送",
                        "显示",
                        "换行",
                        "循环",
                    ]) {
                        ui.collapsing(
                            if lang == i18n::Language::Chinese {
                                "终端与发送"
                            } else {
                                "Terminal & TX"
                            },
                            |ui| {
                            ui.horizontal(|ui| {
                                ui.checkbox(
                                    &mut self.state.ui.send_hex,
                                    if lang == i18n::Language::Chinese {
                                        "HEX 发送"
                                    } else {
                                        "HEX send"
                                    },
                                );
                                ui.checkbox(
                                    &mut self.state.ui.auto_scroll,
                                    if lang == i18n::Language::Chinese {
                                        "自动滚动"
                                    } else {
                                        "Auto scroll"
                                    },
                                );
                                ui.checkbox(
                                    &mut self.state.ui.send_with_newline,
                                    if lang == i18n::Language::Chinese {
                                        "发送附加换行"
                                    } else {
                                        "Append newline"
                                    },
                                );
                            });

                            ui.horizontal(|ui| {
                                ui.label("Display:");
                                ui.selectable_value(
                                    &mut self.state.ui.display_mode,
                                    app::DisplayMode::Hex,
                                    "Hex",
                                );
                                ui.selectable_value(
                                    &mut self.state.ui.display_mode,
                                    app::DisplayMode::Ascii,
                                    "ASCII",
                                );
                                ui.selectable_value(
                                    &mut self.state.ui.display_mode,
                                    app::DisplayMode::Mixed,
                                    "Mixed",
                                );
                            });

                            ui.horizontal(|ui| {
                                ui.label("Newline:");
                                ui.text_edit_singleline(&mut self.state.ui.newline_type);
                            });

                            ui.horizontal(|ui| {
                                ui.checkbox(
                                    &mut self.state.ui.repeat_send,
                                    if lang == i18n::Language::Chinese {
                                        "循环发送"
                                    } else {
                                        "Repeat send"
                                    },
                                );
                                ui.label("Interval(ms):");
                                ui.add(
                                    egui::DragValue::new(&mut self.state.ui.repeat_interval_ms)
                                        .range(50..=60000)
                                        .speed(10),
                                );
                            });
                            },
                        );
                    }

                    if show_group(&[
                        "connection",
                        "network",
                        "tcp",
                        "udp",
                        "reconnect",
                        "连接",
                        "网络",
                        "重连",
                    ]) {
                        ui.collapsing(
                            if lang == i18n::Language::Chinese {
                                "连接与网络"
                            } else {
                                "Connection & Network"
                            },
                            |ui| {
                            ui.horizontal(|ui| {
                                ui.label("TCP Host:");
                                ui.text_edit_singleline(&mut self.state.ui.tcp_host);
                                ui.label("Port:");
                                ui.text_edit_singleline(&mut self.state.ui.tcp_port_text);
                                ui.checkbox(&mut self.state.ui.tcp_is_server, "Server");
                            });

                            ui.horizontal(|ui| {
                                ui.label("UDP Local:");
                                ui.text_edit_singleline(&mut self.state.ui.udp_local_port_text);
                                ui.label("Remote Host:");
                                ui.text_edit_singleline(&mut self.state.ui.udp_remote_host);
                                ui.label("Remote Port:");
                                ui.text_edit_singleline(&mut self.state.ui.udp_remote_port_text);
                            });

                            ui.horizontal(|ui| {
                                ui.checkbox(
                                    &mut self.state.ui.auto_reconnect_enabled,
                                    if lang == i18n::Language::Chinese {
                                        "启用自动重连"
                                    } else {
                                        "Auto reconnect"
                                    },
                                );
                                ui.label(if lang == i18n::Language::Chinese {
                                    "重连间隔(ms)"
                                } else {
                                    "Reconnect interval(ms)"
                                });
                                ui.add(
                                    egui::DragValue::new(
                                        &mut self.state.ui.auto_reconnect_interval_ms,
                                    )
                                    .range(500..=30000)
                                    .speed(50),
                                );
                            });
                            },
                        );
                    }

                    if show_group(&[
                        "protocol",
                        "can",
                        "usb",
                        "analyzer",
                        "parser",
                        "协议",
                        "分析",
                        "解析",
                    ]) {
                        ui.collapsing(
                            if lang == i18n::Language::Chinese {
                                "协议页偏好（CAN / USB / 分析器）"
                            } else {
                                "Protocol Preferences (CAN / USB / Analyzer)"
                            },
                            |ui| {
                            ui.horizontal(|ui| {
                                ui.label("CAN ID:");
                                ui.text_edit_singleline(&mut self.state.ui.can_id_text);
                                ui.label("CAN Data:");
                                ui.text_edit_singleline(&mut self.state.ui.can_data_text);
                            });
                            ui.horizontal(|ui| {
                                ui.checkbox(&mut self.state.ui.can_extended, "CAN Extended");
                                ui.checkbox(&mut self.state.ui.can_fd, "CAN FD");
                                ui.label("CAN bitrate idx:");
                                ui.add(
                                    egui::DragValue::new(&mut self.state.ui.can_bitrate_idx)
                                        .range(0..=8),
                                );
                                ui.label("FD bitrate idx:");
                                ui.add(
                                    egui::DragValue::new(&mut self.state.ui.can_data_bitrate_idx)
                                        .range(0..=7),
                                );
                            });

                            ui.horizontal(|ui| {
                                ui.label("USB protocol idx:");
                                ui.add(
                                    egui::DragValue::new(&mut self.state.ui.usb_protocol_idx)
                                        .range(0..=11),
                                );
                                ui.label("USB speed idx:");
                                ui.add(
                                    egui::DragValue::new(&mut self.state.ui.usb_speed_idx)
                                        .range(0..=4),
                                );
                                ui.label("VID:");
                                ui.text_edit_singleline(&mut self.state.ui.usb_vid_text);
                                ui.label("PID:");
                                ui.text_edit_singleline(&mut self.state.ui.usb_pid_text);
                            });

                            ui.horizontal(|ui| {
                                ui.checkbox(
                                    &mut self.state.ui.parser_auto_parse,
                                    if lang == i18n::Language::Chinese {
                                        "解析器自动解析"
                                    } else {
                                        "Parser auto-parse"
                                    },
                                );
                                ui.checkbox(
                                    &mut self.state.ui.analysis_filter_tx,
                                    "Analyzer TX",
                                );
                                ui.checkbox(
                                    &mut self.state.ui.analysis_filter_rx,
                                    "Analyzer RX",
                                );
                                ui.checkbox(
                                    &mut self.state.ui.analysis_filter_info,
                                    "Analyzer INFO",
                                );
                            });
                            },
                        );
                    }

                    if show_group(&[
                        "ai",
                        "llm",
                        "mcp",
                        "temperature",
                        "智能",
                        "模型",
                    ]) {
                        ui.collapsing(
                            if lang == i18n::Language::Chinese {
                                "AI 与 MCP"
                            } else {
                                "AI & MCP"
                            },
                            |ui| {
                            ui.horizontal(|ui| {
                                ui.label("LLM URL:");
                                ui.text_edit_singleline(&mut self.state.ui.llm_api_url);
                            });
                            ui.horizontal(|ui| {
                                ui.label("LLM Model:");
                                ui.text_edit_singleline(&mut self.state.ui.llm_model_name);
                            });
                            ui.horizontal(|ui| {
                                ui.label("LLM Temp:");
                                ui.text_edit_singleline(&mut self.state.ui.llm_temperature_text);
                                ui.label("MCP Port:");
                                ui.text_edit_singleline(&mut self.state.ui.mcp_port_text);
                            });
                            },
                        );
                    }

                    ui.separator();
                    ui.label(if lang == i18n::Language::Chinese {
                        "说明：偏好将按设定秒数自动保存，且退出时强制保存；支持导入/导出与默认恢复。"
                    } else {
                        "Note: preferences auto-save by configured interval and on exit; import/export/reset are supported."
                    });
                });
            });
            self.show_preferences_dialog = open;
        }

        // ─── 左侧图标条 (Activity Bar) ──────────────────
        egui::SidePanel::left("icon_bar")
            .resizable(false)
            .exact_width(ICON_BAR_WIDTH)
            .frame(
                egui::Frame::new()
                    .fill(if self.state.dark_mode {
                        ICON_BAR_BG
                    } else {
                        egui::Color32::from_rgb(235, 235, 235)
                    })
                    .inner_margin(egui::Margin::symmetric(0, 8)),
            )
            .show(ctx, |ui| {
                let bar_rect = ui.max_rect();
                let gloss_alpha = if self.state.dark_mode { 8 } else { 10 };
                let gloss_rect = egui::Rect::from_min_max(
                    bar_rect.min,
                    egui::pos2(bar_rect.max.x, bar_rect.min.y + 44.0),
                );
                ui.painter().rect_filled(
                    gloss_rect,
                    0.0,
                    egui::Color32::from_white_alpha(gloss_alpha),
                );

                ui.vertical_centered(|ui| {
                    ui.add_space(4.0);
                    // App 图标
                    let (logo_rect, _) =
                        ui.allocate_exact_size(egui::vec2(28.0, 24.0), egui::Sense::hover());
                    draw_app_mark(ui.painter(), logo_rect, self.state.dark_mode);
                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // 导航图标
                    let mut selected_rect = None;
                    for &tab in ActiveTab::all() {
                        let selected = self.state.active_tab == tab;
                        let idx = tab_index(tab);

                        let (item_rect, resp) =
                            ui.allocate_exact_size(egui::vec2(42.0, 38.0), egui::Sense::click());
                        let resp = resp.on_hover_text(tab.label(lang));

                        if resp.clicked() {
                            self.state.active_tab = tab;
                        }

                        self.icon_select_anim[idx] = exp_smooth(
                            self.icon_select_anim[idx],
                            if selected { 1.0 } else { 0.0 },
                            dt,
                            motion.icon_select_speed,
                        );
                        self.icon_hover_anim[idx] = exp_smooth(
                            self.icon_hover_anim[idx],
                            if resp.hovered() { 1.0 } else { 0.0 },
                            dt,
                            motion.icon_hover_speed,
                        );

                        let select_t = ease_apple_out(self.icon_select_anim[idx].clamp(0.0, 1.0));
                        let hover_t =
                            ease_apple_standard(self.icon_hover_anim[idx].clamp(0.0, 1.0));
                        let acrylic_t = (select_t * 0.82 + hover_t * 0.42).min(1.0);

                        let slide_x = motion.icon_slide_px * (select_t * 0.88 + hover_t * 0.28);
                        let slide_y = -(0.8 * select_t + 0.35 * hover_t);
                        let bg_rect = item_rect.shrink2(egui::vec2(1.0, 1.5));

                        let idle_tint = (hover_t * 0.45).clamp(0.0, 1.0);
                        if idle_tint > 0.02 {
                            ui.painter().rect(
                                bg_rect,
                                10.0,
                                if self.state.dark_mode {
                                    egui::Color32::from_rgba_unmultiplied(
                                        95,
                                        112,
                                        132,
                                        alpha_to_u8(12.0 + 20.0 * idle_tint),
                                    )
                                } else {
                                    egui::Color32::from_rgba_unmultiplied(
                                        230,
                                        236,
                                        245,
                                        alpha_to_u8(16.0 + 18.0 * idle_tint),
                                    )
                                },
                                egui::Stroke::new(0.8, egui::Color32::TRANSPARENT),
                                egui::StrokeKind::Middle,
                            );
                        }

                        if acrylic_t > 0.01 {
                            let fill_alpha = alpha_to_u8(motion.acrylic_alpha * acrylic_t);
                            let border_alpha =
                                alpha_to_u8((motion.acrylic_alpha * 0.86) * acrylic_t);

                            let fill_color = if self.state.dark_mode {
                                egui::Color32::from_rgba_unmultiplied(120, 170, 220, fill_alpha)
                            } else {
                                egui::Color32::from_rgba_unmultiplied(220, 236, 255, fill_alpha)
                            };
                            let border_color = if self.state.dark_mode {
                                egui::Color32::from_rgba_unmultiplied(170, 215, 255, border_alpha)
                            } else {
                                egui::Color32::from_rgba_unmultiplied(145, 185, 228, border_alpha)
                            };

                            let depth_alpha =
                                alpha_to_u8((motion.acrylic_alpha * 0.38) * acrylic_t);
                            ui.painter().rect_filled(
                                bg_rect.translate(egui::vec2(0.0, 1.0)),
                                9.0,
                                egui::Color32::from_black_alpha(depth_alpha),
                            );

                            ui.painter().rect(
                                bg_rect,
                                9.0,
                                fill_color,
                                egui::Stroke::new(1.0, border_color),
                                egui::StrokeKind::Middle,
                            );

                            let sheen_progress =
                                (self.acrylic_sheen_phase + idx as f32 * 0.11).fract();
                            let sheen_x = bg_rect.left() - bg_rect.width() * 0.45
                                + sheen_progress * bg_rect.width() * 1.9;
                            let sheen_rect = egui::Rect::from_center_size(
                                egui::pos2(sheen_x, bg_rect.center().y),
                                egui::vec2(bg_rect.width() * 0.34, bg_rect.height() * 0.86),
                            );
                            let sheen_alpha = alpha_to_u8(motion.acrylic_sheen_alpha * acrylic_t);
                            ui.painter().with_clip_rect(bg_rect).rect_filled(
                                sheen_rect,
                                7.0,
                                egui::Color32::from_white_alpha(sheen_alpha),
                            );
                        }

                        let icon_color = if selected {
                            if self.state.dark_mode {
                                egui::Color32::from_rgb(242, 247, 255)
                            } else {
                                egui::Color32::from_rgb(32, 64, 110)
                            }
                        } else if self.state.dark_mode {
                            egui::Color32::from_rgb(155, 162, 173)
                                .gamma_multiply(0.90 + hover_t * 0.22)
                        } else {
                            egui::Color32::from_rgb(110, 115, 126)
                                .gamma_multiply(0.92 + hover_t * 0.20)
                        };

                        draw_sidebar_tab_icon(
                            ui.painter(),
                            tab,
                            egui::pos2(
                                item_rect.center().x + slide_x,
                                item_rect.center().y + slide_y,
                            ),
                            icon_color,
                            16.5,
                        );

                        // 记录选中项矩形，用于绘制平滑活动指示器
                        if selected {
                            selected_rect = Some(item_rect);
                        }

                        ui.add_space(2.0);
                    }

                    if let Some(rect) = selected_rect {
                        let target_y = rect.center().y;
                        let y = if let Some(current) = self.activity_indicator_y {
                            exp_smooth(current, target_y, dt, motion.indicator_speed)
                        } else {
                            target_y
                        };
                        self.activity_indicator_y = Some(y);

                        let indicator_h = (rect.height() * 0.72).max(18.0);
                        let indicator_rect = egui::Rect::from_center_size(
                            egui::pos2(ui.min_rect().left() + 2.5, y),
                            egui::vec2(3.2, indicator_h),
                        );
                        ui.painter().rect_filled(indicator_rect, 1.5, ACCENT_COLOR);
                    }

                    // 底部操作
                    ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                        ui.add_space(8.0);

                        // 主题切换
                        let theme_tip = if self.state.dark_mode {
                            Tr::light_mode(lang)
                        } else {
                            Tr::dark_mode(lang)
                        };
                        let (theme_rect, theme_resp) =
                            ui.allocate_exact_size(egui::vec2(24.0, 22.0), egui::Sense::click());
                        draw_theme_toggle_icon(
                            ui.painter(),
                            theme_rect,
                            self.state.dark_mode,
                            egui::Color32::from_rgb(160, 160, 160),
                        );
                        if theme_resp.clicked() {
                            self.state.dark_mode = !self.state.dark_mode;
                            if self.state.dark_mode {
                                let mut v = egui::Visuals::dark();
                                v.override_text_color =
                                    Some(egui::Color32::from_rgb(220, 220, 230));
                                ctx.set_visuals(v);
                            } else {
                                ctx.set_visuals(egui::Visuals::light());
                            }
                        }
                        theme_resp.on_hover_text(theme_tip);

                        ui.add_space(6.0);

                        // 侧边栏展开/折叠
                        let (expand_rect, expand_resp) =
                            ui.allocate_exact_size(egui::vec2(20.0, 18.0), egui::Sense::click());
                        draw_sidebar_chevron_icon(
                            ui.painter(),
                            expand_rect,
                            self.state.ui.sidebar_expanded,
                            egui::Color32::from_rgb(160, 160, 160),
                        );
                        if expand_resp.clicked() {
                            self.state.ui.sidebar_expanded = !self.state.ui.sidebar_expanded;
                        }
                        expand_resp.on_hover_text("Toggle Sidebar");

                        ui.add_space(6.0);

                        // 连接状态指示灯
                        let connected = self.state.is_any_connected();
                        let dot_color = if connected {
                            egui::Color32::from_rgb(46, 160, 67)
                        } else {
                            egui::Color32::from_rgb(128, 128, 128)
                        };
                        ui.label(egui::RichText::new("\u{2B24}").size(10.0).color(dot_color));
                    });
                });
            });

        // ─── 侧边栏标签区（可折叠） ────────────────────
        if self.sidebar_width_anim > 1.0 {
            egui::SidePanel::left("sidebar_labels")
                .resizable(false)
                .exact_width(self.sidebar_width_anim)
                .frame(
                    egui::Frame::new()
                        .fill(if self.state.dark_mode {
                            SIDEBAR_BG
                        } else {
                            egui::Color32::from_rgb(243, 243, 243)
                        })
                        .inner_margin(egui::Margin::symmetric(8, 12)),
                )
                .show(ctx, |ui| {
                    draw_brand_logo(
                        ui,
                        self.state.dark_mode,
                        self.logo_breathe_phase,
                        sidebar_visibility,
                    );
                    ui.add_space(8.0);

                    // 标题
                    ui.label(
                        egui::RichText::new(Tr::app_title(lang))
                            .size(14.0)
                            .strong()
                            .color(
                                egui::Color32::WHITE.gamma_multiply(sidebar_visibility.max(0.35)),
                            ),
                    );
                    ui.add_space(12.0);

                    let mut last_cat = "";
                    for &tab in ActiveTab::all() {
                        let cat = tab.category();
                        if cat != last_cat {
                            if !last_cat.is_empty() {
                                ui.add_space(8.0);
                            }
                            ui.label(
                                egui::RichText::new(cat).size(10.0).color(
                                    egui::Color32::from_rgb(120, 120, 120)
                                        .gamma_multiply(sidebar_visibility.max(0.35)),
                                ),
                            );
                            ui.add_space(2.0);
                            last_cat = cat;
                        }

                        let selected = self.state.active_tab == tab;
                        let label_text = tab.label(lang);
                        let rt = if selected {
                            egui::RichText::new(label_text).size(13.0).strong().color(
                                if self.state.dark_mode {
                                    egui::Color32::WHITE
                                        .gamma_multiply(sidebar_visibility.max(0.45))
                                } else {
                                    egui::Color32::from_rgb(0, 100, 180)
                                        .gamma_multiply(sidebar_visibility.max(0.45))
                                },
                            )
                        } else {
                            egui::RichText::new(label_text).size(13.0).color(
                                egui::Color32::from_rgb(170, 170, 170)
                                    .gamma_multiply(sidebar_visibility.max(0.35)),
                            )
                        };

                        let resp = ui.selectable_label(selected, rt);
                        if resp.clicked() {
                            self.state.active_tab = tab;
                        }
                    }

                    // 底部信息
                    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                        ui.add_space(4.0);
                        if self.state.is_running {
                            ui.label(
                                egui::RichText::new(format!(
                                    "\u{25B6} {}",
                                    Tr::control_active(lang)
                                ))
                                .size(11.0)
                                .color(egui::Color32::from_rgb(46, 160, 67)),
                            );
                        }
                        ui.label(
                            egui::RichText::new(format!("{}", self.state.active_conn))
                                .size(11.0)
                                .color(
                                    egui::Color32::from_rgb(120, 120, 120)
                                        .gamma_multiply(sidebar_visibility.max(0.35)),
                                ),
                        );
                    });
                });
        }

        // ─── 中央内容区 ─────────────────────────────────
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .inner_margin(egui::Margin::same(16))
                    .fill(if self.state.dark_mode {
                        egui::Color32::from_rgb(30, 30, 35)
                    } else {
                        egui::Color32::from_rgb(255, 255, 255)
                    }),
            )
            .show(ctx, |ui| {
                ui.add_space(tab_slide * 0.24 + content_lift);
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        let content_width = ui.available_width();
                        ui.set_min_width(content_width);

                        match self.state.active_tab {
                            ActiveTab::Dashboard => views::dashboard::show(ui, &mut self.state),
                            ActiveTab::Connections => views::connections::show(ui, &mut self.state),
                            ActiveTab::SerialDebug => {
                                views::serial_debug::show(ui, &mut self.state)
                            }
                            ActiveTab::ProtocolAnalysis => {
                                views::protocol_analysis::show(ui, &mut self.state)
                            }
                            ActiveTab::PacketBuilder => {
                                views::packet_builder::show(ui, &mut self.state)
                            }
                            ActiveTab::Topology => views::topology::show(ui, &mut self.state),
                            ActiveTab::PidControl => views::pid_control::show(ui, &mut self.state),
                            ActiveTab::NnTuning => views::nn_tuning::show(ui, &mut self.state),
                            ActiveTab::DataViz => views::data_viz::show(ui, &mut self.state),
                            ActiveTab::ModbusTools => views::modbus_view::show(ui, &mut self.state),
                            ActiveTab::CanopenTools => {
                                views::canopen_view::show(ui, &mut self.state)
                            }
                        }

                        let fade_alpha = ((1.0 - tab_transition_eased)
                            .powf(motion.content_fade_power)
                            * motion.overlay_alpha) as u8;
                        if fade_alpha > 0 {
                            let overlay = if self.state.dark_mode {
                                egui::Color32::from_rgba_premultiplied(20, 22, 28, fade_alpha)
                            } else {
                                egui::Color32::from_rgba_premultiplied(248, 249, 252, fade_alpha)
                            };
                            ui.painter().rect_filled(ui.max_rect(), 0.0, overlay);
                        }

                        let sweep = tab_transition_soft;
                        if sweep < 1.0 {
                            let rect = ui.max_rect();
                            let left = rect.left() + content_slide * 0.5;
                            let line_w = (rect.width() - content_slide * 0.5).max(24.0)
                                * (0.12 + 0.88 * sweep);
                            let line_rect = egui::Rect::from_min_size(
                                egui::pos2(left, rect.top()),
                                egui::vec2(line_w, 1.5),
                            );
                            ui.painter().rect_filled(
                                line_rect,
                                0.0,
                                egui::Color32::from_rgba_premultiplied(
                                    0,
                                    122,
                                    204,
                                    alpha_to_u8(
                                        motion.content_sweep_alpha
                                            * (1.0 - tab_transition_eased).powf(0.85),
                                    ),
                                ),
                            );
                        }
                    });
            });
    }
}

fn format_bytes_compact(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{}B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1}K", bytes as f64 / 1024.0)
    } else {
        format!("{:.1}M", bytes as f64 / (1024.0 * 1024.0))
    }
}

fn draw_brand_logo(ui: &mut egui::Ui, dark_mode: bool, breathe_phase: f32, visibility: f32) {
    let desired = egui::vec2(120.0, 54.0);
    let (rect, _) = ui.allocate_exact_size(desired, egui::Sense::hover());
    let painter = ui.painter();

    let vis = visibility.max(0.2);
    let breathe = 0.94 + 0.06 * breathe_phase.sin().abs();

    let bg = if dark_mode {
        egui::Color32::from_rgb(34, 40, 54)
    } else {
        egui::Color32::from_rgb(232, 239, 250)
    };
    painter.rect_filled(rect, 10.0, bg.gamma_multiply(vis));

    let icon_rect =
        egui::Rect::from_min_size(rect.min + egui::vec2(8.0, 8.0), egui::vec2(38.0, 38.0));
    let icon_bg = if dark_mode {
        egui::Color32::from_rgb(25, 142, 230)
    } else {
        egui::Color32::from_rgb(18, 126, 214)
    };
    painter.rect_filled(icon_rect, 10.0, icon_bg.gamma_multiply(vis * breathe));
    let c = icon_rect.center();
    painter.circle_stroke(c, 11.0, egui::Stroke::new(2.0, egui::Color32::WHITE));
    painter.line_segment(
        [
            egui::pos2(c.x - 5.2, c.y + 3.8),
            egui::pos2(c.x + 4.8, c.y - 4.6),
        ],
        egui::Stroke::new(2.2, egui::Color32::WHITE),
    );
    painter.circle_filled(egui::pos2(c.x + 6.2, c.y + 6.2), 2.1, egui::Color32::WHITE);

    painter.text(
        rect.min + egui::vec2(54.0, 16.0),
        egui::Align2::LEFT_TOP,
        "ROBOT",
        egui::FontId::proportional(14.0),
        egui::Color32::from_rgb(220, 230, 255).gamma_multiply(vis),
    );
    painter.text(
        rect.min + egui::vec2(54.0, 33.0),
        egui::Align2::LEFT_TOP,
        "CONTROL",
        egui::FontId::proportional(10.0),
        if dark_mode {
            egui::Color32::from_rgb(170, 170, 180).gamma_multiply(vis)
        } else {
            egui::Color32::from_rgb(80, 90, 110).gamma_multiply(vis)
        },
    );
}

fn tab_index(tab: ActiveTab) -> usize {
    ActiveTab::all()
        .iter()
        .position(|&candidate| candidate == tab)
        .unwrap_or(0)
}

fn alpha_to_u8(alpha: f32) -> u8 {
    alpha.clamp(0.0, 255.0).round() as u8
}

fn draw_app_mark(painter: &egui::Painter, rect: egui::Rect, dark_mode: bool) {
    let bg = if dark_mode {
        egui::Color32::from_rgb(22, 140, 226)
    } else {
        egui::Color32::from_rgb(14, 118, 206)
    };
    painter.rect_filled(rect, 8.0, bg);
    let c = rect.center();
    painter.circle_stroke(c, 7.0, egui::Stroke::new(1.5, egui::Color32::WHITE));
    painter.line_segment(
        [
            egui::pos2(c.x - 3.5, c.y + 2.2),
            egui::pos2(c.x + 3.3, c.y - 3.5),
        ],
        egui::Stroke::new(1.8, egui::Color32::WHITE),
    );
    painter.circle_filled(egui::pos2(c.x + 4.5, c.y + 4.5), 1.5, egui::Color32::WHITE);
}

fn draw_theme_toggle_icon(
    painter: &egui::Painter,
    rect: egui::Rect,
    dark_mode: bool,
    color: egui::Color32,
) {
    let c = rect.center();
    if dark_mode {
        painter.circle_stroke(c, 5.0, egui::Stroke::new(1.4, color));
        for i in 0..8 {
            let a = i as f32 * std::f32::consts::TAU / 8.0;
            let p1 = egui::pos2(c.x + a.cos() * 7.0, c.y + a.sin() * 7.0);
            let p2 = egui::pos2(c.x + a.cos() * 9.2, c.y + a.sin() * 9.2);
            painter.line_segment([p1, p2], egui::Stroke::new(1.2, color));
        }
    } else {
        painter.circle_filled(c, 6.2, color);
        painter.circle_filled(
            egui::pos2(c.x + 2.6, c.y - 1.2),
            6.4,
            egui::Color32::from_rgba_premultiplied(0, 0, 0, 180),
        );
    }
}

fn draw_sidebar_chevron_icon(
    painter: &egui::Painter,
    rect: egui::Rect,
    expanded: bool,
    color: egui::Color32,
) {
    let c = rect.center();
    let pts = if expanded {
        vec![
            egui::pos2(c.x + 2.8, c.y - 4.2),
            egui::pos2(c.x - 2.0, c.y),
            egui::pos2(c.x + 2.8, c.y + 4.2),
        ]
    } else {
        vec![
            egui::pos2(c.x - 2.8, c.y - 4.2),
            egui::pos2(c.x + 2.0, c.y),
            egui::pos2(c.x - 2.8, c.y + 4.2),
        ]
    };
    painter.add(egui::Shape::line(pts, egui::Stroke::new(1.5, color)));
}

fn draw_sidebar_tab_icon(
    painter: &egui::Painter,
    tab: ActiveTab,
    center: egui::Pos2,
    color: egui::Color32,
    size: f32,
) {
    let s = size;
    let stroke = egui::Stroke::new(1.55, color);
    match tab {
        ActiveTab::Dashboard => {
            let w = s * 0.14;
            let gap = s * 0.08;
            let x0 = center.x - (w * 2.0 + gap * 1.5);
            let base = center.y + s * 0.34;
            let hs = [s * 0.34, s * 0.56, s * 0.78, s * 0.46];
            for (i, h) in hs.iter().enumerate() {
                let left = x0 + i as f32 * (w + gap);
                let r = egui::Rect::from_min_size(egui::pos2(left, base - *h), egui::vec2(w, *h));
                painter.rect_filled(r, 1.2, color);
            }
        }
        ActiveTab::Connections => {
            painter.circle_stroke(egui::pos2(center.x - s * 0.22, center.y), s * 0.14, stroke);
            painter.circle_stroke(egui::pos2(center.x + s * 0.22, center.y), s * 0.14, stroke);
            painter.line_segment(
                [
                    egui::pos2(center.x - s * 0.08, center.y),
                    egui::pos2(center.x + s * 0.08, center.y),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    egui::pos2(center.x - s * 0.33, center.y + s * 0.15),
                    egui::pos2(center.x + s * 0.33, center.y + s * 0.15),
                ],
                egui::Stroke::new(1.1, color),
            );
        }
        ActiveTab::SerialDebug => {
            let body = egui::Rect::from_center_size(center, egui::vec2(s * 0.62, s * 0.42));
            painter.rect_stroke(body, 2.0, stroke, egui::StrokeKind::Middle);
            painter.line_segment(
                [
                    egui::pos2(body.left() + 2.0, center.y),
                    egui::pos2(body.right() - 2.0, center.y),
                ],
                egui::Stroke::new(1.0, color),
            );
            painter.line_segment(
                [
                    egui::pos2(center.x, body.bottom()),
                    egui::pos2(center.x, body.bottom() + s * 0.18),
                ],
                stroke,
            );
        }
        ActiveTab::ProtocolAnalysis => {
            let body = egui::Rect::from_center_size(center, egui::vec2(s * 0.62, s * 0.44));
            painter.rect_stroke(body, 2.0, stroke, egui::StrokeKind::Middle);
            let p0 = egui::pos2(body.left() + s * 0.05, body.bottom() - s * 0.08);
            let p1 = egui::pos2(center.x - s * 0.12, center.y + s * 0.02);
            let p2 = egui::pos2(center.x + s * 0.02, center.y + s * 0.10);
            let p3 = egui::pos2(body.right() - s * 0.08, body.top() + s * 0.02);
            painter.line_segment([p0, p1], stroke);
            painter.line_segment([p1, p2], stroke);
            painter.line_segment([p2, p3], stroke);
            painter.circle_filled(p1, 1.6, color);
            painter.circle_filled(p2, 1.6, color);
        }
        ActiveTab::PacketBuilder => {
            let r = egui::Rect::from_center_size(center, egui::vec2(s * 0.62, s * 0.48));
            painter.rect_stroke(r, 2.0, stroke, egui::StrokeKind::Middle);
            painter.line_segment(
                [
                    egui::pos2(r.left(), r.top() + s * 0.14),
                    egui::pos2(r.right(), r.top() + s * 0.14),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    egui::pos2(center.x, r.top()),
                    egui::pos2(center.x, r.bottom()),
                ],
                egui::Stroke::new(1.0, color),
            );
        }
        ActiveTab::Topology => {
            let p1 = egui::pos2(center.x, center.y - s * 0.30);
            let p2 = egui::pos2(center.x - s * 0.30, center.y + s * 0.20);
            let p3 = egui::pos2(center.x + s * 0.30, center.y + s * 0.20);
            painter.add(egui::Shape::convex_polygon(
                vec![p1, p2, p3],
                egui::Color32::TRANSPARENT,
                stroke,
            ));
            painter.circle_filled(p1, 1.8, color);
            painter.circle_filled(p2, 1.8, color);
            painter.circle_filled(p3, 1.8, color);
        }
        ActiveTab::PidControl => {
            let r = egui::Rect::from_center_size(center, egui::vec2(s * 0.62, s * 0.42));
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
        ActiveTab::NnTuning => {
            let nodes = [
                egui::pos2(center.x - s * 0.22, center.y - s * 0.10),
                egui::pos2(center.x - s * 0.22, center.y + s * 0.16),
                egui::pos2(center.x + s * 0.02, center.y - s * 0.24),
                egui::pos2(center.x + s * 0.22, center.y + s * 0.02),
            ];
            painter.line_segment([nodes[0], nodes[2]], egui::Stroke::new(1.0, color));
            painter.line_segment([nodes[1], nodes[2]], egui::Stroke::new(1.0, color));
            painter.line_segment([nodes[2], nodes[3]], egui::Stroke::new(1.0, color));
            for p in nodes {
                painter.circle_filled(p, 1.9, color);
            }
        }
        ActiveTab::DataViz => {
            let p0 = egui::pos2(center.x - s * 0.33, center.y + s * 0.24);
            let p1 = egui::pos2(center.x - s * 0.16, center.y + s * 0.06);
            let p2 = egui::pos2(center.x + s * 0.01, center.y + s * 0.13);
            let p3 = egui::pos2(center.x + s * 0.24, center.y - s * 0.18);
            painter.line_segment([p0, p1], stroke);
            painter.line_segment([p1, p2], stroke);
            painter.line_segment([p2, p3], stroke);
            painter.line_segment(
                [
                    egui::pos2(center.x - s * 0.35, center.y + s * 0.28),
                    egui::pos2(center.x + s * 0.30, center.y + s * 0.28),
                ],
                egui::Stroke::new(1.0, color),
            );
        }
        ActiveTab::ModbusTools => {
            let r = egui::Rect::from_center_size(center, egui::vec2(s * 0.64, s * 0.44));
            painter.rect_stroke(r, 2.0, stroke, egui::StrokeKind::Middle);
            for row in 0..2 {
                for col in 0..3 {
                    let cell = egui::Rect::from_min_size(
                        egui::pos2(
                            r.left() + 3.0 + col as f32 * (r.width() - 6.0) / 3.0,
                            r.top() + 3.0 + row as f32 * (r.height() - 6.0) / 2.0,
                        ),
                        egui::vec2((r.width() - 10.0) / 3.0, (r.height() - 9.0) / 2.0),
                    );
                    painter.rect_stroke(
                        cell,
                        1.0,
                        egui::Stroke::new(0.9, color),
                        egui::StrokeKind::Middle,
                    );
                }
            }
        }
        ActiveTab::CanopenTools => {
            let r = egui::Rect::from_center_size(center, egui::vec2(s * 0.64, s * 0.46));
            painter.rect_stroke(r, 2.0, stroke, egui::StrokeKind::Middle);

            let left = egui::pos2(r.left() + s * 0.10, center.y);
            let mid = egui::pos2(center.x, r.top() + s * 0.10);
            let right = egui::pos2(r.right() - s * 0.10, center.y);
            let bot = egui::pos2(center.x, r.bottom() - s * 0.10);

            painter.line_segment([left, mid], egui::Stroke::new(1.1, color));
            painter.line_segment([mid, right], egui::Stroke::new(1.1, color));
            painter.line_segment([right, bot], egui::Stroke::new(1.1, color));
            painter.line_segment([bot, left], egui::Stroke::new(1.1, color));

            for p in [left, mid, right, bot] {
                painter.circle_filled(p, 1.8, color);
            }
        }
    }
}

fn draw_motion_profile_chip(ui: &mut egui::Ui, level: MotionLevel, dark_mode: bool) {
    let profile = level.profile();
    let chip_bg = if dark_mode {
        egui::Color32::from_rgba_premultiplied(255, 255, 255, 20)
    } else {
        egui::Color32::from_rgba_premultiplied(255, 255, 255, 34)
    };
    let chip_stroke = if dark_mode {
        egui::Color32::from_rgba_premultiplied(255, 255, 255, 36)
    } else {
        egui::Color32::from_rgba_premultiplied(0, 0, 0, 30)
    };

    let resp = egui::Frame::new()
        .fill(chip_bg)
        .stroke(egui::Stroke::new(1.0, chip_stroke))
        .corner_radius(6.0)
        .inner_margin(egui::Margin::symmetric(6, 3))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                draw_motion_metric(ui, "流", profile.smoothness);
                ui.add_space(4.0);
                draw_motion_metric(ui, "质", profile.texture);
                ui.add_space(4.0);
                draw_motion_metric(ui, "效", profile.efficiency);
            });
        })
        .response;

    resp.on_hover_text(format!(
        "动效画像（{}）\n流畅度: {}/10\n质感: {}/10\n性能: {}/10",
        level.label(),
        profile.smoothness,
        profile.texture,
        profile.efficiency
    ));
}

fn draw_motion_metric(ui: &mut egui::Ui, short_label: &str, value: u8) {
    ui.label(
        egui::RichText::new(short_label)
            .size(10.5)
            .color(egui::Color32::from_rgb(228, 236, 248)),
    );
    let (rect, _) = ui.allocate_exact_size(egui::vec2(18.0, 5.0), egui::Sense::hover());
    ui.painter().rect_filled(
        rect,
        2.0,
        egui::Color32::from_rgba_premultiplied(255, 255, 255, 34),
    );
    let progress = (value as f32 / 10.0).clamp(0.0, 1.0);
    let fill_rect = egui::Rect::from_min_max(
        rect.min,
        egui::pos2(rect.left() + rect.width() * progress, rect.bottom()),
    );
    ui.painter()
        .rect_filled(fill_rect, 2.0, egui::Color32::from_rgb(166, 221, 255));
}

fn exp_smooth(current: f32, target: f32, dt: f32, speed: f32) -> f32 {
    let alpha = 1.0 - (-speed * dt).exp();
    current + (target - current) * alpha
}

fn ease_apple_out(t: f32) -> f32 {
    cubic_bezier_ease(t, 0.22, 1.0, 0.36, 1.0)
}

fn ease_apple_standard(t: f32) -> f32 {
    cubic_bezier_ease(t, 0.25, 0.10, 0.25, 1.0)
}

fn cubic_bezier_ease(t: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let target_x = t.clamp(0.0, 1.0);
    let mut u = target_x;

    for _ in 0..5 {
        let x = cubic_bezier_component(u, x1, x2);
        let dx = cubic_bezier_derivative(u, x1, x2);
        if dx.abs() < 1e-5 {
            break;
        }
        u = (u - (x - target_x) / dx).clamp(0.0, 1.0);
    }

    cubic_bezier_component(u, y1, y2)
}

fn cubic_bezier_component(t: f32, p1: f32, p2: f32) -> f32 {
    let omt = 1.0 - t;
    3.0 * omt * omt * t * p1 + 3.0 * omt * t * t * p2 + t * t * t
}

fn cubic_bezier_derivative(t: f32, p1: f32, p2: f32) -> f32 {
    3.0 * (1.0 - t).powi(2) * p1 + 6.0 * (1.0 - t) * t * (p2 - p1) + 3.0 * t.powi(2) * (1.0 - p2)
}

fn generate_app_icon() -> egui::IconData {
    let w = 64u32;
    let h = 64u32;
    let mut rgba = vec![0u8; (w * h * 4) as usize];

    for y in 0..h {
        for x in 0..w {
            let idx = ((y * w + x) * 4) as usize;
            let dx = x as f32 - 32.0;
            let dy = y as f32 - 32.0;
            let r2 = dx * dx + dy * dy;

            let inside = r2 <= 29.5 * 29.5;
            let (r, g, b, a) = if inside {
                let grad = (1.0 - (r2.sqrt() / 29.5)).clamp(0.0, 1.0);
                (
                    (12.0 + 16.0 * grad) as u8,
                    (118.0 + 42.0 * grad) as u8,
                    (206.0 + 36.0 * grad) as u8,
                    255u8,
                )
            } else {
                (0, 0, 0, 0)
            };
            rgba[idx] = r;
            rgba[idx + 1] = g;
            rgba[idx + 2] = b;
            rgba[idx + 3] = a;
        }
    }

    // minimalist white ring + slash + dot
    let paint_dot = |buf: &mut [u8], cx: i32, cy: i32, rr: i32, color: [u8; 4]| {
        for yy in -rr..=rr {
            for xx in -rr..=rr {
                if xx * xx + yy * yy <= rr * rr {
                    let x = cx + xx;
                    let y = cy + yy;
                    if x >= 0 && y >= 0 && x < w as i32 && y < h as i32 {
                        let idx = (((y as u32) * w + (x as u32)) * 4) as usize;
                        buf[idx..idx + 4].copy_from_slice(&color);
                    }
                }
            }
        }
    };
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let dx = x - 32;
            let dy = y - 32;
            let d2 = dx * dx + dy * dy;
            if (12 * 12..=14 * 14).contains(&d2) {
                let idx = (((y as u32) * w + (x as u32)) * 4) as usize;
                rgba[idx..idx + 4].copy_from_slice(&[255, 255, 255, 255]);
            }
        }
    }
    for i in -8..=8 {
        let x = 32 + i;
        let y = 32 - i - 1;
        if x >= 0 && y >= 0 && x < w as i32 && y < h as i32 {
            let idx = (((y as u32) * w + (x as u32)) * 4) as usize;
            rgba[idx..idx + 4].copy_from_slice(&[255, 255, 255, 255]);
        }
    }
    paint_dot(&mut rgba, 43, 43, 3, [255, 255, 255, 255]);

    egui::IconData {
        rgba,
        width: w,
        height: h,
    }
}

/// 跨平台加载系统 CJK 字体
/// Windows: 微软雅黑 / 黑体 / 宋体
/// macOS:   苹方 / STHeiti
/// Linux:   Noto Sans CJK / 文泉驿
fn load_cjk_font_data() -> Option<Vec<u8>> {
    #[cfg(target_os = "windows")]
    let candidates: &[&str] = &[
        "C:\\Windows\\Fonts\\msyh.ttc",
        "C:\\Windows\\Fonts\\msyhbd.ttc",
        "C:\\Windows\\Fonts\\simhei.ttf",
        "C:\\Windows\\Fonts\\simsun.ttc",
        "C:\\Windows\\Fonts\\malgun.ttf",
    ];

    #[cfg(target_os = "macos")]
    let candidates: &[&str] = &[
        "/System/Library/Fonts/PingFang.ttc",
        "/Library/Fonts/Arial Unicode.ttf",
        "/System/Library/Fonts/STHeiti Light.ttc",
        "/System/Library/Fonts/STHeiti Medium.ttc",
        "/System/Library/Fonts/Hiragino Sans GB.ttc",
        "/System/Library/Fonts/Supplemental/Songti.ttc",
        "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
    ];

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let candidates: &[&str] = &[
        // Noto CJK (Debian/Ubuntu)
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansCJKSC-Regular.otf",
        "/usr/share/fonts/truetype/noto/NotoSansCJKsc-Regular.otf",
        // Noto CJK (Fedora/RHEL)
        "/usr/share/fonts/google-noto-cjk/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/google-noto-cjk/NotoSansCJKsc-Regular.otf",
        // Noto CJK (Arch Linux)
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/noto-cjk/NotoSansCJKSC-Regular.otf",
        // Noto CJK (openSUSE)
        "/usr/share/fonts/truetype/NotoSansCJK-Regular.ttc",
        // 文泉驿
        "/usr/share/fonts/wenquanyi/wqy-microhei/wqy-microhei.ttc",
        "/usr/share/fonts/wenquanyi/wqy-zenhei/wqy-zenhei.ttc",
        "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
        "/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc",
        // Droid
        "/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf",
        // NixOS (通过 fonts.fontDir.enable)
        "/run/current-system/sw/share/X11/fonts/NotoSansCJK-Regular.ttc",
    ];

    for path in candidates {
        if let Ok(data) = std::fs::read(path) {
            log::info!("Loaded CJK font: {}", path);
            return Some(data);
        }
    }
    log::warn!("No CJK font found on system. Chinese text may not render correctly.");
    None
}
