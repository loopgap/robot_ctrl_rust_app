use eframe::egui;

use crate::i18n::Language;
use crate::settings::{load_preferences, save_preferences, AppPreferences};
use crate::theme::ACCENT_COLOR;
use crate::tools::{
    base64_workshop::Base64Tool, checksum::ChecksumTool, json_workshop::JsonTool,
    log_inspector::LogTool, time_converter::TimeConverterTool, url_codec::UrlCodecTool,
    uuid_batch::UuidBatchTool,
};
use crate::workflow::render_loop_panel;

#[derive(Clone, Copy, PartialEq, Eq)]
enum ActiveTool {
    Checksum,
    Json,
    Log,
    UrlCodec,
    TimeConverter,
    Base64,
    UuidBatch,
}

impl ActiveTool {
    fn all() -> [Self; 7] {
        [
            Self::Checksum,
            Self::Json,
            Self::Log,
            Self::UrlCodec,
            Self::TimeConverter,
            Self::Base64,
            Self::UuidBatch,
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
        }
    }
}

pub struct ToolSuiteApp {
    language: Language,
    active: ActiveTool,
    checksum: ChecksumTool,
    json: JsonTool,
    log: LogTool,
    url_codec: UrlCodecTool,
    time_converter: TimeConverterTool,
    base64_tool: Base64Tool,
    uuid_batch: UuidBatchTool,
}

impl Default for ToolSuiteApp {
    fn default() -> Self {
        let prefs = load_preferences();
        Self {
            language: prefs.language,
            active: ActiveTool::Checksum,
            checksum: ChecksumTool::default(),
            json: JsonTool::default(),
            log: LogTool::default(),
            url_codec: UrlCodecTool::default(),
            time_converter: TimeConverterTool::default(),
            base64_tool: Base64Tool::default(),
            uuid_batch: UuidBatchTool::default(),
        }
    }
}

impl ToolSuiteApp {
    fn persist_preferences(&self) {
        save_preferences(&AppPreferences {
            language: self.language,
        });
    }
}

impl Drop for ToolSuiteApp {
    fn drop(&mut self) {
        self.persist_preferences();
    }
}

impl eframe::App for ToolSuiteApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let old_language = self.language;
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.heading(self.language.tr(
                    "Rust Micro Tools Suite · 中文",
                    "Rust Micro Tools Suite · English",
                ));
                ui.colored_label(
                    ACCENT_COLOR,
                    self.language.tr(
                        "市场高频小工具 · 闭环流程",
                        "Practical tools with closed-loop workflow",
                    ),
                );
                egui::ComboBox::from_label(self.language.tr("语言", "Language"))
                    .selected_text(self.language.label())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.language, Language::Zh, "中文");
                        ui.selectable_value(&mut self.language, Language::En, "English");
                    });
            });
            ui.separator();
            ui.horizontal(|ui| {
                for tab in ActiveTool::all() {
                    let selected = self.active == tab;
                    let button = egui::Button::new(tab.label(self.language)).selected(selected);
                    if ui.add(button).clicked() {
                        self.active = tab;
                    }
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.columns(2, |cols| {
                cols[0].set_min_width(840.0);
                cols[1].set_min_width(360.0);

                match self.active {
                    ActiveTool::Checksum => self.checksum.ui(&mut cols[0], ctx, self.language),
                    ActiveTool::Json => self.json.ui(&mut cols[0], ctx, self.language),
                    ActiveTool::Log => self.log.ui(&mut cols[0], ctx, self.language),
                    ActiveTool::UrlCodec => self.url_codec.ui(&mut cols[0], ctx, self.language),
                    ActiveTool::TimeConverter => {
                        self.time_converter.ui(&mut cols[0], ctx, self.language)
                    }
                    ActiveTool::Base64 => self.base64_tool.ui(&mut cols[0], ctx, self.language),
                    ActiveTool::UuidBatch => self.uuid_batch.ui(&mut cols[0], ctx, self.language),
                }

                let steps = match self.active {
                    ActiveTool::Checksum => self.checksum.loop_steps(self.language),
                    ActiveTool::Json => self.json.loop_steps(self.language),
                    ActiveTool::Log => self.log.loop_steps(self.language),
                    ActiveTool::UrlCodec => self.url_codec.loop_steps(self.language),
                    ActiveTool::TimeConverter => self.time_converter.loop_steps(self.language),
                    ActiveTool::Base64 => self.base64_tool.loop_steps(self.language),
                    ActiveTool::UuidBatch => self.uuid_batch.loop_steps(self.language),
                };
                render_loop_panel(&mut cols[1], &steps, self.language);
            });
        });

        if old_language != self.language {
            self.persist_preferences();
        }
    }
}
