use eframe::egui;
use std::path::{Path, PathBuf};

use crate::file_ops::{open_text_file, save_text_file};
use crate::i18n::Language;
use crate::settings::{load_preferences, save_preferences, AppPreferences};
use crate::theme::{apply_theme, ACCENT_COLOR};
use crate::tools::base64_workshop::Base64Tool;
use crate::tools::checksum::ChecksumTool;
use crate::tools::csv_cleaner::CsvCleanerTool;
use crate::tools::json_workshop::JsonTool;
use crate::tools::jwt_inspector::JwtInspectorTool;
use crate::tools::log_inspector::LogTool;
use crate::tools::regex_workbench::RegexWorkbenchTool;
use crate::tools::time_converter::TimeConverterTool;
use crate::tools::url_codec::UrlCodecTool;
use crate::tools::uuid_batch::UuidBatchTool;
use crate::workflow::{render_loop_panel, LoopStep};

const DOCS_FALLBACK_URL: &str =
    "https://github.com/loopgap/robot_ctrl_rust_app/blob/main/docs/src/README.md";
const WIDE_LAYOUT_MIN_WIDTH: f32 = 1180.0;
const TAB_BUTTONS_MIN_WIDTH: f32 = 1240.0;
const MIN_UI_SCALE_PERCENT: u32 = 100;
const MAX_UI_SCALE_PERCENT: u32 = 220;
const DEFAULT_UI_SCALE_PERCENT: u32 = 150;
const UI_SCALE_STEP_PERCENT: i32 = 10;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ResponsiveMode {
    Wide,
    Compact,
}

fn responsive_mode(width: f32) -> ResponsiveMode {
    if width >= WIDE_LAYOUT_MIN_WIDTH {
        ResponsiveMode::Wide
    } else {
        ResponsiveMode::Compact
    }
}

fn path_to_file_url(path: &Path) -> String {
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let mut raw = canonical.to_string_lossy().replace('\\', "/");
    if let Some(stripped) = raw.strip_prefix("//?/") {
        raw = stripped.to_string();
    }
    if !raw.starts_with('/') {
        raw = format!("/{raw}");
    }
    let escaped = raw
        .replace('%', "%25")
        .replace(' ', "%20")
        .replace('#', "%23")
        .replace('?', "%3F");
    format!("file://{escaped}")
}

fn resolve_docs_url() -> String {
    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            candidates.push(exe_dir.join("help_index.html"));
            candidates.push(exe_dir.join("help").join("index.html"));
            candidates.push(exe_dir.join("docs").join("index.html"));
            for ancestor in exe_dir.ancestors().take(4) {
                candidates.push(ancestor.join("help_index.html"));
                candidates.push(ancestor.join("docs").join("help").join("index.html"));
                candidates.push(ancestor.join("docs").join("index.html"));
                candidates.push(ancestor.join("docs").join("book").join("index.html"));
                candidates.push(ancestor.join("docs").join("site").join("index.html"));
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        candidates.push(PathBuf::from("/usr/share/rust-tools-suite/help_index.html"));
        candidates.push(PathBuf::from("/usr/share/rust-tools-suite/docs/index.html"));
        candidates.push(PathBuf::from("/usr/share/rust-tools-suite/docs/book/index.html"));
        candidates.push(PathBuf::from("/usr/share/doc/rust-tools-suite/help_index.html"));
        candidates.push(PathBuf::from("/usr/share/doc/rust-tools-suite/docs/index.html"));
        candidates.push(PathBuf::from("/usr/share/doc/rust-tools-suite/docs/book/index.html"));
    }

    candidates.push(PathBuf::from("help_index.html"));
    candidates.push(PathBuf::from("docs").join("help").join("index.html"));
    candidates.push(PathBuf::from("docs").join("index.html"));
    candidates.push(PathBuf::from("docs").join("book").join("index.html"));
    candidates.push(PathBuf::from("docs").join("site").join("index.html"));

    candidates
        .into_iter()
        .find(|path| path.exists())
        .map(|path| path_to_file_url(&path))
        .unwrap_or_else(|| DOCS_FALLBACK_URL.to_string())
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ActiveTool {
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
}

impl ActiveTool {
    fn all() -> [Self; 10] {
        [
            Self::Checksum,
            Self::Json,
            Self::Log,
            Self::UrlCodec,
            Self::TimeConverter,
            Self::Base64,
            Self::UuidBatch,
            Self::CsvCleaner,
            Self::JwtInspector,
            Self::RegexWorkbench,
        ]
    }

    fn label(self, language: Language) -> String {
        match self {
            Self::Checksum => language.tr("校验和", "Checksum"),
            Self::Json => language.tr("JSON 工坊", "JSON Workshop"),
            Self::Log => language.tr("日志巡检", "Log Inspector"),
            Self::UrlCodec => language.tr("URL 编解码", "URL Codec"),
            Self::TimeConverter => language.tr("时间戳转换", "Timestamp Converter"),
            Self::Base64 => language.tr("Base64 工坊", "Base64 Workshop"),
            Self::UuidBatch => language.tr("UUID 批量生成", "UUID Batch Generator"),
            Self::CsvCleaner => language.tr("CSV 清洗", "CSV Cleaner"),
            Self::JwtInspector => language.tr("JWT 解析", "JWT Inspector"),
            Self::RegexWorkbench => language.tr("Regex 巡检", "Regex Workbench"),
        }
    }

    fn storage_key(self) -> &'static str {
        match self {
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
        }
    }

    fn from_storage_key(value: &str) -> Self {
        match value {
            "json" => Self::Json,
            "log" => Self::Log,
            "url_codec" => Self::UrlCodec,
            "time_converter" => Self::TimeConverter,
            "base64" => Self::Base64,
            "uuid_batch" => Self::UuidBatch,
            "csv_cleaner" => Self::CsvCleaner,
            "jwt_inspector" => Self::JwtInspector,
            "regex_workbench" => Self::RegexWorkbench,
            _ => Self::Checksum,
        }
    }
}

pub struct ToolSuiteApp {
    language: Language,
    dark_mode: bool,
    ui_scale_percent: u32,
    pending_ui_scale_percent: u32,
    applied_dark_mode: Option<bool>,
    applied_ui_scale_percent: Option<u32>,
    workflow_drawer_open: bool,
    active: ActiveTool,
    show_preferences: bool,
    show_about: bool,
    show_shortcuts: bool,
    status_message: String,
    checksum: ChecksumTool,
    json: JsonTool,
    log: LogTool,
    url_codec: UrlCodecTool,
    time_converter: TimeConverterTool,
    base64_tool: Base64Tool,
    uuid_batch: UuidBatchTool,
    csv_cleaner: CsvCleanerTool,
    jwt_inspector: JwtInspectorTool,
    regex_workbench: RegexWorkbenchTool,
}

impl Default for ToolSuiteApp {
    fn default() -> Self {
        let prefs = load_preferences().unwrap_or_default();
        Self {
            language: prefs.language,
            dark_mode: prefs.dark_mode,
            ui_scale_percent: prefs
                .ui_scale_percent
                .clamp(MIN_UI_SCALE_PERCENT, MAX_UI_SCALE_PERCENT),
            pending_ui_scale_percent: prefs
                .ui_scale_percent
                .clamp(MIN_UI_SCALE_PERCENT, MAX_UI_SCALE_PERCENT),
            applied_dark_mode: None,
            applied_ui_scale_percent: None,
            workflow_drawer_open: prefs.workflow_drawer_open,
            active: ActiveTool::from_storage_key(&prefs.active_tool_key),
            show_preferences: false,
            show_about: false,
            show_shortcuts: false,
            status_message: String::new(),
            checksum: ChecksumTool::default(),
            json: JsonTool::default(),
            log: LogTool::default(),
            url_codec: UrlCodecTool::default(),
            time_converter: TimeConverterTool::default(),
            base64_tool: Base64Tool::default(),
            uuid_batch: UuidBatchTool::default(),
            csv_cleaner: CsvCleanerTool::default(),
            jwt_inspector: JwtInspectorTool::default(),
            regex_workbench: RegexWorkbenchTool::default(),
        }
    }
}

impl ToolSuiteApp {
    fn persist_preferences(&self) {
        let _ = save_preferences(&AppPreferences {
            language: self.language,
            dark_mode: self.dark_mode,
            ui_scale_percent: self
                .ui_scale_percent
                .clamp(MIN_UI_SCALE_PERCENT, MAX_UI_SCALE_PERCENT),
            workflow_drawer_open: self.workflow_drawer_open,
            active_tool_key: self.active.storage_key().to_string(),
        });
    }

    fn apply_ui_scale(&self, ctx: &egui::Context) {
        let scale = self
            .ui_scale_percent
            .clamp(MIN_UI_SCALE_PERCENT, MAX_UI_SCALE_PERCENT) as f32
            / 100.0;
        ctx.set_pixels_per_point(scale);
    }

    fn ensure_theme(&mut self, ctx: &egui::Context) {
        if self.applied_dark_mode == Some(self.dark_mode) {
            return;
        }
        apply_theme(ctx, self.dark_mode);
        self.applied_dark_mode = Some(self.dark_mode);
    }

    fn ensure_ui_scale(&mut self, ctx: &egui::Context) {
        let current = self
            .ui_scale_percent
            .clamp(MIN_UI_SCALE_PERCENT, MAX_UI_SCALE_PERCENT);
        if self.applied_ui_scale_percent == Some(current) {
            return;
        }
        self.apply_ui_scale(ctx);
        self.applied_ui_scale_percent = Some(current);
    }

    fn effective_repaint_interval_ms(&self, ctx: &egui::Context) -> u64 {
        let (minimized, focused) = ctx.input(|i| (i.viewport().minimized, i.viewport().focused));
        let mut interval = 16_u64;

        if minimized.unwrap_or(false) {
            interval = 500;
        } else if !focused.unwrap_or(true) {
            interval = 125;
        }

        interval
    }

    fn ui_scale_status(&self, percent: u32) -> String {
        match self.language {
            Language::Zh => format!("界面缩放已调整为 {}%", percent),
            Language::En => format!("UI scale set to {}%", percent),
        }
    }

    fn apply_pending_ui_scale(&mut self) {
        let scale = self
            .pending_ui_scale_percent
            .clamp(MIN_UI_SCALE_PERCENT, MAX_UI_SCALE_PERCENT);
        self.ui_scale_percent = scale;
        self.pending_ui_scale_percent = scale;
        self.applied_ui_scale_percent = None;
        self.set_status(self.ui_scale_status(scale));
    }

    fn reset_ui_scale(&mut self) {
        self.pending_ui_scale_percent = DEFAULT_UI_SCALE_PERCENT;
        self.apply_pending_ui_scale();
    }

    fn step_ui_scale(&mut self, delta_percent: i32) {
        let current = self.ui_scale_percent as i32;
        let next = (current + delta_percent)
            .clamp(MIN_UI_SCALE_PERCENT as i32, MAX_UI_SCALE_PERCENT as i32)
            as u32;
        if next != self.ui_scale_percent {
            self.ui_scale_percent = next;
            self.pending_ui_scale_percent = next;
            self.applied_ui_scale_percent = None;
            self.set_status(self.ui_scale_status(next));
        }
    }

    fn set_status(&mut self, message: impl Into<String>) {
        self.status_message = message.into();
    }

    fn active_steps(&self) -> Vec<LoopStep> {
        match self.active {
            ActiveTool::Checksum => self.checksum.loop_steps(self.language),
            ActiveTool::Json => self.json.loop_steps(self.language),
            ActiveTool::Log => self.log.loop_steps(self.language),
            ActiveTool::UrlCodec => self.url_codec.loop_steps(self.language),
            ActiveTool::TimeConverter => self.time_converter.loop_steps(self.language),
            ActiveTool::Base64 => self.base64_tool.loop_steps(self.language),
            ActiveTool::UuidBatch => self.uuid_batch.loop_steps(self.language),
            ActiveTool::CsvCleaner => self.csv_cleaner.loop_steps(self.language),
            ActiveTool::JwtInspector => self.jwt_inspector.loop_steps(self.language),
            ActiveTool::RegexWorkbench => self.regex_workbench.loop_steps(self.language),
        }
    }

    fn active_output_text(&self) -> Option<String> {
        match self.active {
            ActiveTool::Checksum => self.checksum.output_text(),
            ActiveTool::Json => self.json.output_text(),
            ActiveTool::Log => self.log.output_text(),
            ActiveTool::UrlCodec => self.url_codec.output_text(),
            ActiveTool::TimeConverter => self.time_converter.output_text(),
            ActiveTool::Base64 => self.base64_tool.output_text(),
            ActiveTool::UuidBatch => self.uuid_batch.output_text(),
            ActiveTool::CsvCleaner => self.csv_cleaner.output_text(),
            ActiveTool::JwtInspector => self.jwt_inspector.output_text(),
            ActiveTool::RegexWorkbench => self.regex_workbench.output_text(),
        }
    }

    fn clear_active_tool(&mut self) {
        match self.active {
            ActiveTool::Checksum => self.checksum.clear(),
            ActiveTool::Json => self.json.clear(),
            ActiveTool::Log => self.log.clear(),
            ActiveTool::UrlCodec => self.url_codec.clear(),
            ActiveTool::TimeConverter => self.time_converter.clear(),
            ActiveTool::Base64 => self.base64_tool.clear(),
            ActiveTool::UuidBatch => self.uuid_batch.clear(),
            ActiveTool::CsvCleaner => self.csv_cleaner.clear(),
            ActiveTool::JwtInspector => self.jwt_inspector.clear(),
            ActiveTool::RegexWorkbench => self.regex_workbench.clear(),
        }
    }

    fn active_tool_supports_input_file(&self) -> bool {
        !matches!(self.active, ActiveTool::UuidBatch)
    }

    fn load_input_for_active(&mut self, text: String) {
        match self.active {
            ActiveTool::Checksum => self.checksum.load_input(text),
            ActiveTool::Json => self.json.load_input(text),
            ActiveTool::Log => self.log.load_input(text),
            ActiveTool::UrlCodec => self.url_codec.load_input(text),
            ActiveTool::TimeConverter => self.time_converter.load_input(text),
            ActiveTool::Base64 => self.base64_tool.load_input(text),
            ActiveTool::UuidBatch => {}
            ActiveTool::CsvCleaner => self.csv_cleaner.load_input(text),
            ActiveTool::JwtInspector => self.jwt_inspector.load_input(text),
            ActiveTool::RegexWorkbench => self.regex_workbench.load_input(text),
        }
    }

    fn load_sample_for_active(&mut self) {
        let sample = match self.active {
            ActiveTool::Checksum => "sensor_id=gyro-7\nvalue=42.15\nstatus=ok\n".to_string(),
            ActiveTool::Json => {
                r#"{"device":"arm-01","enabled":true,"limits":{"current":12,"speed":48}}"#
                    .to_string()
            }
            ActiveTool::Log => {
                "INFO startup ok\nWARN retry link=serial\nERROR timeout on COM3\n".to_string()
            }
            ActiveTool::UrlCodec => {
                "redirect=https://example.com/callback?token=a+b&name=robot control".to_string()
            }
            ActiveTool::TimeConverter => "1712044800".to_string(),
            ActiveTool::Base64 => "robot-control-suite".to_string(),
            ActiveTool::UuidBatch => return,
            ActiveTool::CsvCleaner => "id,name\n1,Alice\n1,Alice\n2,Bob\n".to_string(),
            ActiveTool::JwtInspector => "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJkZW1vIiwic2NvcGUiOiJyZWFkOnN0YXR1cyJ9.c2ln".to_string(),
            ActiveTool::RegexWorkbench => "ERROR timeout\nINFO ready\nERROR invalid crc\n".to_string(),
        };
        self.load_input_for_active(sample);
        self.set_status(self.language.tr("已加载示例输入", "Loaded sample input"));
    }

    fn active_default_output_file_name(&self) -> &'static str {
        match self.active {
            ActiveTool::Checksum => "checksum-result.txt",
            ActiveTool::Json => "formatted.json",
            ActiveTool::Log => "filtered.log",
            ActiveTool::UrlCodec => "url-output.txt",
            ActiveTool::TimeConverter => "time-output.txt",
            ActiveTool::Base64 => "base64-output.txt",
            ActiveTool::UuidBatch => "uuid-output.txt",
            ActiveTool::CsvCleaner => "cleaned.csv",
            ActiveTool::JwtInspector => "jwt-output.txt",
            ActiveTool::RegexWorkbench => "regex-output.txt",
        }
    }

    fn open_input_file_for_active(&mut self) {
        if !self.active_tool_supports_input_file() {
            self.set_status(self.language.tr(
                "当前工具不支持导入输入文件",
                "The active tool does not support input files",
            ));
            return;
        }

        match open_text_file() {
            Ok(Some((path, text))) => {
                self.load_input_for_active(text);
                self.set_status(match self.language {
                    Language::Zh => format!("已导入文件：{}", path.display()),
                    Language::En => format!("Imported file: {}", path.display()),
                });
            }
            Ok(None) => {}
            Err(err) => self.set_status(err),
        }
    }

    fn save_output_file_for_active(&mut self) {
        let Some(output) = self.active_output_text() else {
            self.set_status(
                self.language
                    .tr("当前没有可导出的结果", "No output to export"),
            );
            return;
        };

        match save_text_file(self.active_default_output_file_name(), &output) {
            Ok(Some(path)) => {
                self.set_status(match self.language {
                    Language::Zh => format!("结果已保存：{}", path.display()),
                    Language::En => format!("Saved output: {}", path.display()),
                });
            }
            Ok(None) => {}
            Err(err) => self.set_status(err),
        }
    }

    fn copy_output_for_active(&mut self, ctx: &egui::Context) {
        let Some(output) = self.active_output_text() else {
            self.set_status(
                self.language
                    .tr("当前没有可复制的结果", "No output to copy"),
            );
            return;
        };
        ctx.copy_text(output);
        self.set_status(self.language.tr("已复制当前结果", "Copied active output"));
    }

    fn render_active_tool(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| match self.active {
                ActiveTool::Checksum => self.checksum.ui(ui, ctx, self.language),
                ActiveTool::Json => self.json.ui(ui, ctx, self.language),
                ActiveTool::Log => self.log.ui(ui, ctx, self.language),
                ActiveTool::UrlCodec => self.url_codec.ui(ui, ctx, self.language),
                ActiveTool::TimeConverter => self.time_converter.ui(ui, ctx, self.language),
                ActiveTool::Base64 => self.base64_tool.ui(ui, ctx, self.language),
                ActiveTool::UuidBatch => self.uuid_batch.ui(ui, ctx, self.language),
                ActiveTool::CsvCleaner => self.csv_cleaner.ui(ui, ctx, self.language),
                ActiveTool::JwtInspector => self.jwt_inspector.ui(ui, ctx, self.language),
                ActiveTool::RegexWorkbench => self.regex_workbench.ui(ui, ctx, self.language),
            });
    }

    fn render_tool_selector(&mut self, ui: &mut egui::Ui, available_width: f32) {
        if available_width >= TAB_BUTTONS_MIN_WIDTH {
            ui.horizontal_wrapped(|ui| {
                for tab in ActiveTool::all() {
                    let selected = self.active == tab;
                    let button = egui::Button::new(tab.label(self.language))
                        .selected(selected)
                        .min_size(egui::vec2(148.0, 34.0));
                    if ui.add(button).clicked() {
                        self.active = tab;
                    }
                }
            });
        } else {
            egui::ComboBox::from_id_salt("tool_selector")
                .width(280.0)
                .selected_text(self.active.label(self.language))
                .show_ui(ui, |ui| {
                    for tab in ActiveTool::all() {
                        ui.selectable_value(&mut self.active, tab, tab.label(self.language));
                    }
                });
        }
    }

    fn render_menu_bar(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, mode: ResponsiveMode) {
        let language = self.language;

        ui.menu_button(language.tr("文件", "File"), |ui| {
            if ui
                .add_enabled(
                    self.active_tool_supports_input_file(),
                    egui::Button::new(language.tr("导入输入文件", "Open Input File")),
                )
                .clicked()
            {
                self.open_input_file_for_active();
                ui.close_menu();
            }

            if ui.button(language.tr("加载示例", "Load Sample")).clicked() {
                self.load_sample_for_active();
                ui.close_menu();
            }

            if ui
                .button(language.tr("另存为结果", "Save Output As"))
                .clicked()
            {
                self.save_output_file_for_active();
                ui.close_menu();
            }

            if ui.button(language.tr("偏好设置", "Preferences")).clicked() {
                self.show_preferences = true;
                ui.close_menu();
            }

            ui.separator();
            if ui.button(language.tr("退出", "Quit")).clicked() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                ui.close_menu();
            }
        });

        ui.menu_button(language.tr("编辑", "Edit"), |ui| {
            if ui
                .button(language.tr("复制当前结果", "Copy Output"))
                .clicked()
            {
                self.copy_output_for_active(ctx);
                ui.close_menu();
            }
            if ui
                .button(language.tr("清空当前工具", "Clear Active Tool"))
                .clicked()
            {
                self.clear_active_tool();
                self.set_status(language.tr("已清空当前工具", "Cleared active tool"));
                ui.close_menu();
            }
        });

        ui.menu_button(language.tr("视图", "View"), |ui| {
            let workflow_label = if self.workflow_drawer_open {
                language.tr("隐藏流程面板", "Hide Workflow Panel")
            } else {
                match mode {
                    ResponsiveMode::Wide => language.tr("显示流程侧栏", "Show Workflow Side Panel"),
                    ResponsiveMode::Compact => language.tr("打开流程抽屉", "Open Workflow Drawer"),
                }
            };
            if ui.button(workflow_label).clicked() {
                self.workflow_drawer_open = !self.workflow_drawer_open;
                ui.close_menu();
            }

            let theme_label = if self.dark_mode {
                language.tr("切换到浅色主题", "Switch to Light Theme")
            } else {
                language.tr("切换到深色主题", "Switch to Dark Theme")
            };
            if ui.button(theme_label).clicked() {
                self.dark_mode = !self.dark_mode;
                ui.close_menu();
            }

            ui.separator();
            ui.label(language.tr(
                "界面缩放（调整后点应用）",
                "UI Scale (apply after adjusting)",
            ));
            ui.label(format!(
                "{}: {}%",
                language.tr("当前生效", "Current"),
                self.ui_scale_percent
            ));
            ui.add(
                egui::Slider::new(
                    &mut self.pending_ui_scale_percent,
                    MIN_UI_SCALE_PERCENT..=MAX_UI_SCALE_PERCENT,
                )
                .suffix("%"),
            );
            ui.horizontal_wrapped(|ui| {
                if ui.button(language.tr("应用缩放", "Apply Scale")).clicked() {
                    self.apply_pending_ui_scale();
                    ui.close_menu();
                }
                if ui
                    .button(language.tr("重置为 150%", "Reset to 150%"))
                    .clicked()
                {
                    self.reset_ui_scale();
                    ui.close_menu();
                }
            });
            if self.pending_ui_scale_percent != self.ui_scale_percent {
                ui.small(language.tr(
                    "拖动滑块只修改待应用值，点击“应用缩放”后才真正生效。",
                    "Dragging changes only the pending value. Click Apply Scale to commit it.",
                ));
            }
            ui.small(language.tr("快捷缩放：Ctrl + 滚轮", "Quick zoom: Ctrl + mouse wheel"));
        });

        ui.menu_button(language.tr("工具", "Tools"), |ui| {
            for tab in ActiveTool::all() {
                if ui
                    .selectable_label(self.active == tab, tab.label(language))
                    .clicked()
                {
                    self.active = tab;
                    ui.close_menu();
                }
            }
        });

        ui.menu_button(language.tr("帮助", "Help"), |ui| {
            if ui.button(language.tr("关于", "About")).clicked() {
                self.show_about = true;
                ui.close_menu();
            }
            if ui.button(language.tr("快捷键", "Shortcuts")).clicked() {
                self.show_shortcuts = true;
                ui.close_menu();
            }
            if ui.button(language.tr("文档", "Documentation")).clicked() {
                ctx.open_url(egui::OpenUrl {
                    url: resolve_docs_url(),
                    new_tab: true,
                });
                ui.close_menu();
            }
        });

        ui.menu_button(language.tr("语言", "Language"), |ui| {
            if ui
                .selectable_label(self.language == Language::Zh, "中文")
                .clicked()
            {
                self.language = Language::Zh;
                ui.close_menu();
            }
            if ui
                .selectable_label(self.language == Language::En, "English")
                .clicked()
            {
                self.language = Language::En;
                ui.close_menu();
            }
        });
    }

    fn render_dialogs(&mut self, ctx: &egui::Context) {
        let language = self.language;

        if self.show_preferences {
            let mut open = self.show_preferences;
            egui::Window::new(language.tr("偏好设置", "Preferences"))
                .open(&mut open)
                .resizable(false)
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(language.tr("语言", "Language"));
                        ui.selectable_value(&mut self.language, Language::Zh, "中文");
                        ui.selectable_value(&mut self.language, Language::En, "English");
                    });
                    ui.checkbox(&mut self.dark_mode, language.tr("深色主题", "Dark Theme"));
                    ui.checkbox(
                        &mut self.workflow_drawer_open,
                        language.tr("默认展开流程面板", "Open workflow panel by default"),
                    );
                    ui.label(format!(
                        "{}: {}%",
                        language.tr("当前生效", "Current"),
                        self.ui_scale_percent
                    ));
                    ui.add(
                        egui::Slider::new(
                            &mut self.pending_ui_scale_percent,
                            MIN_UI_SCALE_PERCENT..=MAX_UI_SCALE_PERCENT,
                        )
                        .text(language.tr("界面缩放", "UI Scale"))
                        .suffix("%"),
                    );
                    ui.horizontal_wrapped(|ui| {
                        if ui.button(language.tr("应用缩放", "Apply Scale")).clicked() {
                            self.apply_pending_ui_scale();
                        }
                        if ui
                            .button(language.tr("重置为 150%", "Reset to 150%"))
                            .clicked()
                        {
                            self.reset_ui_scale();
                        }
                    });
                    ui.small(
                        language.tr("快捷缩放：Ctrl + 滚轮", "Quick zoom: Ctrl + mouse wheel"),
                    );
                });
            self.show_preferences = open;
        }

        if self.show_about {
            let mut open = self.show_about;
            egui::Window::new(language.tr("关于", "About"))
                .open(&mut open)
                .resizable(false)
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.heading(language.tr("Rust Tools Suite", "Rust Tools Suite"));
                    ui.label(format!(
                        "{}: {}",
                        language.tr("版本", "Version"),
                        env!("CARGO_PKG_VERSION")
                    ));
                    ui.label(format!(
                        "{}: {}",
                        language.tr("语言", "Language"),
                        self.language.label()
                    ));
                    ui.separator();
                    ui.label(language.tr(
                        "统一聚合 10 款高频工具，提供响应式布局、闭环流程面板、文件导入导出与双语体验。",
                        "A unified desktop suite for 10 high-frequency tools with responsive layout, workflow guidance, file I/O, and bilingual UX.",
                    ));
                });
            self.show_about = open;
        }

        if self.show_shortcuts {
            let mut open = self.show_shortcuts;
            egui::Window::new(language.tr("快捷键", "Shortcuts"))
                .open(&mut open)
                .resizable(false)
                .collapsible(false)
                .show(ctx, |ui| {
                    let shortcuts = match language {
                        Language::Zh => vec![
                            ("Ctrl+O", "导入输入文件"),
                            ("Ctrl+Shift+S", "另存为结果"),
                            ("Ctrl+K", "清空当前工具"),
                            ("Ctrl+Shift+L", "切换语言"),
                            ("F1", "打开快捷键窗口"),
                        ],
                        Language::En => vec![
                            ("Ctrl+O", "Open input file"),
                            ("Ctrl+Shift+S", "Save output as"),
                            ("Ctrl+K", "Clear active tool"),
                            ("Ctrl+Shift+L", "Toggle language"),
                            ("F1", "Open shortcuts window"),
                        ],
                    };

                    egui::Grid::new("shortcuts_grid")
                        .num_columns(2)
                        .spacing([16.0, 8.0])
                        .show(ui, |ui| {
                            for (key, desc) in shortcuts {
                                ui.monospace(key);
                                ui.label(desc);
                                ui.end_row();
                            }
                        });
                });
            self.show_shortcuts = open;
        }
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let open_shortcut = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::O);
        if ctx.input_mut(|i| i.consume_shortcut(&open_shortcut)) {
            self.open_input_file_for_active();
        }

        let mut save_modifiers = egui::Modifiers::COMMAND;
        save_modifiers.shift = true;
        let save_shortcut = egui::KeyboardShortcut::new(save_modifiers, egui::Key::S);
        if ctx.input_mut(|i| i.consume_shortcut(&save_shortcut)) {
            self.save_output_file_for_active();
        }

        let clear_shortcut = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::K);
        if ctx.input_mut(|i| i.consume_shortcut(&clear_shortcut)) {
            self.clear_active_tool();
            self.set_status(self.language.tr("已清空当前工具", "Cleared active tool"));
        }

        let mut language_modifiers = egui::Modifiers::COMMAND;
        language_modifiers.shift = true;
        let language_shortcut = egui::KeyboardShortcut::new(language_modifiers, egui::Key::L);
        if ctx.input_mut(|i| i.consume_shortcut(&language_shortcut)) {
            self.language = self.language.toggle();
        }

        if ctx.input(|i| i.key_pressed(egui::Key::F1)) {
            self.show_shortcuts = true;
        }

        let zoom_delta = ctx.input(|i| {
            if i.modifiers.ctrl {
                i.raw_scroll_delta.y
            } else {
                0.0
            }
        });
        if zoom_delta.abs() > f32::EPSILON {
            self.step_ui_scale(if zoom_delta > 0.0 {
                UI_SCALE_STEP_PERCENT
            } else {
                -UI_SCALE_STEP_PERCENT
            });
        }
    }
}

impl Drop for ToolSuiteApp {
    fn drop(&mut self) {
        self.persist_preferences();
    }
}

impl eframe::App for ToolSuiteApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.ensure_theme(ctx);
        self.ensure_ui_scale(ctx);
        self.handle_shortcuts(ctx);
        let width = ctx.available_rect().width();
        let mode = responsive_mode(width);

        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                self.render_menu_bar(ui, ctx, mode);
                ui.separator();
                ui.vertical(|ui| {
                    ui.heading(
                        self.language
                            .tr("Rust Tools Suite · 中文", "Rust Tools Suite · English"),
                    );
                    ui.colored_label(
                        ACCENT_COLOR,
                        self.language.tr(
                            "10 款工具统一管理 · 响应式桌面工作流",
                            "Unified management for 10 tools · responsive desktop workflow",
                        ),
                    );
                });
            });
            ui.separator();
            self.render_tool_selector(ui, width);
        });

        if mode == ResponsiveMode::Wide && self.workflow_drawer_open {
            egui::SidePanel::right("workflow_side_panel")
                .default_width(360.0)
                .min_width(300.0)
                .resizable(true)
                .show(ctx, |ui| {
                    render_loop_panel(ui, &self.active_steps(), self.language);
                });
        }

        if mode == ResponsiveMode::Compact && self.workflow_drawer_open {
            egui::TopBottomPanel::bottom("workflow_drawer")
                .default_height(240.0)
                .min_height(180.0)
                .resizable(true)
                .show(ctx, |ui| {
                    render_loop_panel(ui, &self.active_steps(), self.language);
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_active_tool(ui, ctx);
        });

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(format!(
                    "{}: {}",
                    self.language.tr("当前工具", "Active Tool"),
                    self.active.label(self.language)
                ));
                if !self.status_message.is_empty() {
                    ui.separator();
                    ui.label(format!(
                        "{}: {}",
                        self.language.tr("状态", "Status"),
                        self.status_message
                    ));
                }
                ui.separator();
                ui.label(match mode {
                    ResponsiveMode::Wide => self.language.tr("布局：宽屏", "Layout: wide"),
                    ResponsiveMode::Compact => self
                        .language
                        .tr("布局：紧凑 + 流程抽屉", "Layout: compact + drawer"),
                });
            });
        });

        self.render_dialogs(ctx);
        ctx.request_repaint_after(std::time::Duration::from_millis(
            self.effective_repaint_interval_ms(ctx),
        ));
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.persist_preferences();
    }
}

#[cfg(test)]
mod tests {
    use super::{responsive_mode, ResponsiveMode, WIDE_LAYOUT_MIN_WIDTH};

    #[test]
    fn test_responsive_mode_breakpoints() {
        assert_eq!(
            responsive_mode(WIDE_LAYOUT_MIN_WIDTH - 1.0),
            ResponsiveMode::Compact
        );
        assert_eq!(responsive_mode(WIDE_LAYOUT_MIN_WIDTH), ResponsiveMode::Wide);
    }
}
