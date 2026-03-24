use chrono::{DateTime, Local, LocalResult, NaiveDateTime, TimeZone, Utc};
use eframe::egui;

use crate::guide::render_guide;
use crate::i18n::Language;
use crate::theme::ACCENT_COLOR;
use crate::workflow::{LoopState, LoopStep};

#[derive(Default)]
pub struct TimeConverterTool {
    timestamp_input: String,
    datetime_input: String,
    output: String,
    detail: String,
    executed: bool,
    verified: bool,
    exported: bool,
}

impl TimeConverterTool {
    pub fn ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, lang: Language) {
        ui.heading(lang.tr("时间戳转换", "Timestamp Converter"));
        ui.label(lang.tr(
            "用于日志对时与时区排障。",
            "For log time alignment and timezone troubleshooting.",
        ));
        render_guide(
            ui,
            lang,
            "时间戳转换",
            "Timestamp Converter",
            &[
                (
                    "输入 Unix 时间戳或时间字符串",
                    "Enter Unix timestamp or datetime",
                ),
                ("点击对应转换按钮", "Click corresponding conversion button"),
                (
                    "检查 UTC/Local 与秒毫秒输出",
                    "Verify UTC/local and sec/ms output",
                ),
                ("复制结果给日志平台", "Copy output to log platform"),
            ],
        );
        ui.separator();

        ui.label(lang.tr("Unix 时间戳（秒/毫秒）", "Unix Timestamp (sec/ms)"));
        ui.text_edit_singleline(&mut self.timestamp_input);
        if ui
            .add(
                egui::Button::new(lang.tr("时间戳 → 时间", "Timestamp → Datetime"))
                    .fill(ACCENT_COLOR),
            )
            .clicked()
        {
            self.timestamp_to_datetime(lang);
        }

        ui.separator();
        ui.label(lang.tr(
            "时间字符串（YYYY-MM-DD HH:MM:SS）",
            "Datetime String (YYYY-MM-DD HH:MM:SS)",
        ));
        ui.text_edit_singleline(&mut self.datetime_input);
        if ui
            .button(lang.tr("时间 → 时间戳", "Datetime → Timestamp"))
            .clicked()
        {
            self.datetime_to_timestamp(lang);
        }

        if !self.detail.is_empty() {
            let color = if self.verified {
                egui::Color32::LIGHT_GREEN
            } else {
                egui::Color32::LIGHT_RED
            };
            ui.colored_label(color, &self.detail);
        }

        ui.separator();
        ui.label(lang.tr("输出", "Output"));
        ui.add_sized(
            [ui.available_width(), 180.0],
            egui::TextEdit::multiline(&mut self.output).hint_text("转换结果"),
        );

        if ui.button(lang.tr("复制输出", "Copy Output")).clicked() {
            ctx.copy_text(self.output.clone());
            self.exported = !self.output.is_empty();
        }
    }

    pub fn loop_steps(&self, lang: Language) -> Vec<LoopStep> {
        let has_input =
            !self.timestamp_input.trim().is_empty() || !self.datetime_input.trim().is_empty();
        vec![
            LoopStep {
                name: if matches!(lang, Language::Zh) {
                    "输入"
                } else {
                    "Input"
                },
                state: if has_input {
                    LoopState::Done
                } else {
                    LoopState::Pending
                },
                detail: if has_input {
                    lang.tr("已输入时间戳或时间字符串", "Input captured")
                } else {
                    lang.tr("等待输入转换源数据", "Waiting for source input")
                },
            },
            LoopStep {
                name: if matches!(lang, Language::Zh) {
                    "校验"
                } else {
                    "Validate"
                },
                state: if self.verified {
                    LoopState::Done
                } else if self.executed {
                    LoopState::Warning
                } else {
                    LoopState::Pending
                },
                detail: if self.detail.is_empty() {
                    lang.tr("执行后校验时间格式", "Validate format after run")
                } else {
                    self.detail.clone()
                },
            },
            LoopStep {
                name: if matches!(lang, Language::Zh) {
                    "执行"
                } else {
                    "Execute"
                },
                state: if self.executed {
                    LoopState::Done
                } else {
                    LoopState::Pending
                },
                detail: if self.executed {
                    lang.tr("已完成时间转换", "Conversion done")
                } else {
                    lang.tr("点击任一转换按钮", "Click one conversion")
                },
            },
            LoopStep {
                name: if matches!(lang, Language::Zh) {
                    "验证"
                } else {
                    "Verify"
                },
                state: if self.verified {
                    LoopState::Done
                } else if self.executed {
                    LoopState::Warning
                } else {
                    LoopState::Pending
                },
                detail: if self.verified {
                    lang.tr("结果可用于日志比对", "Ready for log correlation")
                } else {
                    lang.tr("请检查输入格式", "Check input format")
                },
            },
            LoopStep {
                name: if matches!(lang, Language::Zh) {
                    "导出"
                } else {
                    "Export"
                },
                state: if self.exported {
                    LoopState::Done
                } else {
                    LoopState::Pending
                },
                detail: if self.exported {
                    lang.tr("已复制结果", "Copied")
                } else {
                    lang.tr("复制给日志平台/工单系统", "Copy to log/ticket system")
                },
            },
        ]
    }

    fn timestamp_to_datetime(&mut self, lang: Language) {
        self.executed = true;
        self.exported = false;

        let text = self.timestamp_input.trim();
        let parsed = text.parse::<i64>();
        let raw = match parsed {
            Ok(v) => v,
            Err(err) => {
                self.verified = false;
                self.output.clear();
                self.detail = format!(
                    "{}: {err}",
                    lang.tr("时间戳解析失败", "Timestamp parse failed")
                );
                return;
            }
        };

        let dt_utc = if text.len() >= 13 {
            DateTime::<Utc>::from_timestamp_millis(raw)
        } else {
            DateTime::<Utc>::from_timestamp(raw, 0)
        };

        match dt_utc {
            Some(value) => {
                let dt_local = value.with_timezone(&Local);
                self.output = format!(
                    "UTC:   {}\nLocal: {}",
                    value.format("%Y-%m-%d %H:%M:%S"),
                    dt_local.format("%Y-%m-%d %H:%M:%S")
                );
                self.verified = true;
                self.detail = lang.tr("时间戳转换成功", "Timestamp converted");
            }
            None => {
                self.output.clear();
                self.verified = false;
                self.detail = lang.tr("时间戳超出可转换范围", "Out of range");
            }
        }
    }

    fn datetime_to_timestamp(&mut self, lang: Language) {
        self.executed = true;
        self.exported = false;

        let text = self.datetime_input.trim();
        match NaiveDateTime::parse_from_str(text, "%Y-%m-%d %H:%M:%S") {
            Ok(naive) => match Local.from_local_datetime(&naive) {
                LocalResult::Single(dt) => {
                    self.output = format!(
                        "seconds: {}\nmillis:  {}",
                        dt.timestamp(),
                        dt.timestamp_millis()
                    );
                    self.verified = true;
                    self.detail = lang.tr("时间字符串转换成功", "Datetime converted");
                }
                _ => {
                    self.output.clear();
                    self.verified = false;
                    self.detail = lang.tr(
                        "本地时区时间无效或存在歧义",
                        "Invalid or ambiguous local time",
                    );
                }
            },
            Err(err) => {
                self.output.clear();
                self.verified = false;
                self.detail = format!(
                    "{}: {err}",
                    lang.tr("时间格式错误", "Datetime format error")
                );
            }
        }
    }
}
