use eframe::egui;
use regex::Regex;

use crate::guide::render_guide;
use crate::i18n::Language;
use crate::theme::ACCENT_COLOR;
use crate::workflow::{LoopState, LoopStep};

#[derive(Default)]
pub struct LogTool {
    input: String,
    include_regex: String,
    exclude_regex: String,
    output: String,
    matched_count: usize,
    executed: bool,
    verified: bool,
    exported: bool,
    detail: String,
}

impl LogTool {
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn load_input(&mut self, text: String) {
        self.input = text;
        self.output.clear();
        self.matched_count = 0;
        self.executed = false;
        self.verified = false;
        self.exported = false;
        self.detail.clear();
    }

    pub fn output_text(&self) -> Option<String> {
        if self.output.trim().is_empty() {
            None
        } else {
            Some(self.output.clone())
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, lang: Language) {
        ui.heading(lang.tr("日志巡检", "Log Inspector"));
        ui.label(lang.tr(
            "用于故障排查和告警复核。",
            "For troubleshooting and on-call validation.",
        ));
        render_guide(
            ui,
            lang,
            "日志巡检",
            "Log Inspector",
            &[
                ("粘贴原始日志", "Paste raw logs"),
                ("设置包含/排除正则", "Set include/exclude regex"),
                ("执行筛选并检查命中统计", "Run filter and check hits"),
                ("复制结果到工单系统", "Copy result to ticketing system"),
            ],
        );
        ui.separator();

        ui.label(lang.tr("输入日志", "Input Logs"));
        ui.add_sized(
            [ui.available_width(), 180.0],
            egui::TextEdit::multiline(&mut self.input).hint_text("粘贴应用日志"),
        );

        ui.horizontal_wrapped(|ui| {
            ui.label(lang.tr("包含正则", "Include Regex"));
            ui.text_edit_singleline(&mut self.include_regex);
        });
        ui.horizontal_wrapped(|ui| {
            ui.label(lang.tr("排除正则", "Exclude Regex"));
            ui.text_edit_singleline(&mut self.exclude_regex);
        });

        if ui
            .add(egui::Button::new(lang.tr("执行筛选", "Run Filter")).fill(ACCENT_COLOR))
            .clicked()
        {
            self.execute(lang);
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
        ui.label(match lang {
            Language::Zh => format!("筛选结果（{} 行）", self.matched_count),
            Language::En => format!("Filtered Output ({} lines)", self.matched_count),
        });
        ui.add_sized(
            [ui.available_width(), 220.0],
            egui::TextEdit::multiline(&mut self.output).hint_text("筛选结果"),
        );

        if ui
            .button(lang.tr("复制筛选结果", "Copy Filtered Output"))
            .clicked()
        {
            ctx.copy_text(self.output.clone());
            self.exported = !self.output.is_empty();
        }
    }

    pub fn loop_steps(&self, lang: Language) -> Vec<LoopStep> {
        let has_input = !self.input.trim().is_empty();
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
                    format!("{} 行日志", self.input.lines().count())
                } else {
                    lang.tr("等待输入日志", "Waiting for log input")
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
                    lang.tr("待执行后验证正则规则", "Validate regex after run")
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
                    lang.tr("已完成筛选", "Filter done")
                } else {
                    lang.tr("点击执行筛选", "Click run filter")
                },
            },
            LoopStep {
                name: if matches!(lang, Language::Zh) {
                    "验证"
                } else {
                    "Verify"
                },
                state: if self.executed && self.matched_count > 0 {
                    LoopState::Done
                } else if self.executed {
                    LoopState::Warning
                } else {
                    LoopState::Pending
                },
                detail: if self.executed {
                    format!("可疑或目标日志：{} 行", self.matched_count)
                } else {
                    lang.tr("执行后生成命中统计", "Hit stats after run")
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
                    lang.tr("已复制筛选结果", "Copied")
                } else {
                    lang.tr(
                        "可复制结果给工单或报警平台",
                        "Copy to ticket/alert platform",
                    )
                },
            },
        ]
    }

    fn execute(&mut self, lang: Language) {
        self.executed = true;
        self.exported = false;

        let include = if self.include_regex.trim().is_empty() {
            None
        } else {
            match Regex::new(self.include_regex.trim()) {
                Ok(r) => Some(r),
                Err(err) => {
                    self.detail =
                        format!("{}: {err}", lang.tr("包含规则错误", "Include regex error"));
                    self.verified = false;
                    self.output.clear();
                    self.matched_count = 0;
                    return;
                }
            }
        };

        let exclude = if self.exclude_regex.trim().is_empty() {
            None
        } else {
            match Regex::new(self.exclude_regex.trim()) {
                Ok(r) => Some(r),
                Err(err) => {
                    self.detail =
                        format!("{}: {err}", lang.tr("排除规则错误", "Exclude regex error"));
                    self.verified = false;
                    self.output.clear();
                    self.matched_count = 0;
                    return;
                }
            }
        };

        let mut lines = Vec::new();
        for line in self.input.lines() {
            let include_ok = include.as_ref().is_none_or(|r| r.is_match(line));
            let exclude_ok = exclude.as_ref().is_none_or(|r| !r.is_match(line));
            if include_ok && exclude_ok {
                lines.push(line.to_string());
            }
        }

        self.matched_count = lines.len();
        self.output = lines.join("\n");
        self.verified = true;
        self.detail = match lang {
            Language::Zh => format!("筛选完成：命中 {} 行", self.matched_count),
            Language::En => format!("Filter completed: {} lines matched", self.matched_count),
        };
    }
}
