use eframe::egui;

use crate::guide::render_guide;
use crate::i18n::Language;
use crate::theme::ACCENT_COLOR;
use crate::workflow::{LoopState, LoopStep};

#[derive(Default)]
pub struct JsonTool {
    input: String,
    output: String,
    executed: bool,
    verified: bool,
    exported: bool,
    detail: String,
}

impl JsonTool {
    pub fn ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, lang: Language) {
        ui.heading(lang.tr("JSON 工坊", "JSON Workshop"));
        ui.label(lang.tr(
            "用于配置治理与回归排查。",
            "For configuration governance and regression checks.",
        ));
        render_guide(
            ui,
            lang,
            "JSON 工坊",
            "JSON Workshop",
            &[
                ("粘贴 JSON 内容", "Paste JSON content"),
                ("点击格式化或压缩", "Format or compact"),
                ("根据提示修复语法错误", "Fix syntax errors if any"),
                (
                    "复制结果用于部署或联调",
                    "Copy output for deployment/integration",
                ),
            ],
        );
        ui.separator();

        ui.label(lang.tr("输入 JSON", "Input JSON"));
        ui.add_sized(
            [ui.available_width(), 220.0],
            egui::TextEdit::multiline(&mut self.input).hint_text("粘贴 JSON 内容"),
        );

        ui.horizontal(|ui| {
            if ui
                .add(egui::Button::new(lang.tr("格式化", "Format")).fill(ACCENT_COLOR))
                .clicked()
            {
                self.format_pretty(lang);
            }
            if ui.button(lang.tr("压缩", "Compact")).clicked() {
                self.compact(lang);
            }
        });

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
            [ui.available_width(), 220.0],
            egui::TextEdit::multiline(&mut self.output).hint_text("输出结果"),
        );

        if ui.button(lang.tr("复制输出", "Copy Output")).clicked() {
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
                    format!("{} chars", self.input.chars().count())
                } else {
                    lang.tr("等待输入 JSON", "Waiting for JSON input")
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
                    lang.tr("待执行后生成校验结果", "Run to validate")
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
                    lang.tr("已执行格式化/压缩", "Operation done")
                } else {
                    lang.tr("点击格式化或压缩", "Click a transform")
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
                    lang.tr("输出可直接用于配置分发", "Ready for distribution")
                } else {
                    lang.tr("请修复 JSON 语法错误", "Fix JSON syntax errors")
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
                    lang.tr("复制结果到部署或工单系统", "Copy to ticket/deploy")
                },
            },
        ]
    }

    fn process_json(&mut self, lang: Language, pretty: bool) {
        match serde_json::from_str::<serde_json::Value>(&self.input) {
            Ok(val) => {
                let res = if pretty {
                    serde_json::to_string_pretty(&val)
                } else {
                    serde_json::to_string(&val)
                };
                match res {
                    Ok(out) => {
                        self.output = out;
                        self.executed = true;
                        self.verified = true;
                        self.exported = false;
                        self.detail = if pretty {
                            lang.tr("JSON 合法，已完成格式化", "Valid JSON, formatted")
                        } else {
                            lang.tr("JSON 合法，已完成压缩", "Valid JSON, compacted")
                        };
                    }
                    Err(err) => {
                        let op = if pretty { "格式化" } else { "压缩" };
                        self.detail = format!("{op}失败：{err}");
                        self.verified = false;
                    }
                }
            }
            Err(err) => {
                self.output.clear();
                self.executed = true;
                self.verified = false;
                self.exported = false;
                self.detail = format!("{}: {err}", lang.tr("JSON 非法", "Invalid JSON"));
            }
        }
    }

    fn format_pretty(&mut self, lang: Language) {
        self.process_json(lang, true);
    }

    fn compact(&mut self, lang: Language) {
        self.process_json(lang, false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::Language;

    #[test]
    fn test_json_format_pretty() {
        let mut tool = JsonTool {
            input: r#"{"a":1}"#.to_string(),
            ..Default::default()
        };
        tool.process_json(Language::Zh, true);
        assert!(tool.verified);
        assert_eq!(tool.output, "{\n  \"a\": 1\n}");
    }

    #[test]
    fn test_json_compact() {
        let mut tool = JsonTool {
            input: "{\n  \"a\": 1\n}".to_string(),
            ..Default::default()
        };
        tool.process_json(Language::Zh, false);
        assert!(tool.verified);
        assert_eq!(tool.output, r#"{"a":1}"#);
    }
}
