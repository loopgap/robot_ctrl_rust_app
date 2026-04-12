use base64::{engine::general_purpose, Engine};
use eframe::egui;

use crate::guide::render_guide;
use crate::i18n::Language;
use crate::theme::ACCENT_COLOR;
use crate::workflow::{LoopState, LoopStep};

#[derive(Default)]
pub struct Base64Tool {
    input: String,
    output: String,
    use_urlsafe: bool,
    detail: String,
    executed: bool,
    verified: bool,
    exported: bool,
}

impl Base64Tool {
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn load_input(&mut self, text: String) {
        self.input = text;
        self.output.clear();
        self.detail.clear();
        self.executed = false;
        self.verified = false;
        self.exported = false;
    }

    pub fn output_text(&self) -> Option<String> {
        if self.output.trim().is_empty() {
            None
        } else {
            Some(self.output.clone())
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, lang: Language) {
        ui.heading(lang.tr("Base64 工坊", "Base64 Workshop"));
        ui.label(lang.tr(
            "用于 Token、二进制片段与接口字段的编码排障。",
            "For token/binary payload encoding troubleshooting.",
        ));
        render_guide(
            ui,
            lang,
            "Base64 工坊",
            "Base64 Workshop",
            &[
                ("输入待处理文本", "Input text to process"),
                (
                    "选择标准或 URL Safe 模式",
                    "Choose standard or URL-safe mode",
                ),
                ("执行编码或解码", "Run encode or decode"),
                ("复制输出给上下游系统", "Copy output to downstream systems"),
            ],
        );
        ui.separator();

        ui.checkbox(
            &mut self.use_urlsafe,
            lang.tr("使用 URL Safe 模式", "Use URL-safe mode"),
        );
        ui.label(lang.tr("输入", "Input"));
        ui.add_sized(
            [ui.available_width(), 150.0],
            egui::TextEdit::multiline(&mut self.input).hint_text(lang.tr(
                "粘贴原始文本或 Base64 字符串",
                "Paste raw text or Base64 string",
            )),
        );

        ui.horizontal_wrapped(|ui| {
            if ui
                .add(egui::Button::new(lang.tr("编码", "Encode")).fill(ACCENT_COLOR))
                .clicked()
            {
                self.encode(lang);
            }
            if ui.button(lang.tr("解码", "Decode")).clicked() {
                self.decode(lang);
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
            [ui.available_width(), 150.0],
            egui::TextEdit::multiline(&mut self.output),
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
                    lang.tr("等待输入", "Waiting for input")
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
                    lang.tr("执行后生成校验信息", "Validation info after run")
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
                    lang.tr("已执行编码/解码", "Encode/decode done")
                } else {
                    lang.tr("点击编码或解码", "Click encode or decode")
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
                    lang.tr("结果可用于接口联调", "Output ready for integration")
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
                    lang.tr("复制结果到文档或工单", "Copy output to docs/tickets")
                },
            },
        ]
    }

    fn engine(&self) -> &'static general_purpose::GeneralPurpose {
        if self.use_urlsafe {
            &general_purpose::URL_SAFE
        } else {
            &general_purpose::STANDARD
        }
    }

    fn encode(&mut self, lang: Language) {
        self.output = self.engine().encode(self.input.as_bytes());
        self.executed = true;
        self.verified = true;
        self.exported = false;
        self.detail = lang.tr("编码成功", "Encode success");
    }

    fn decode(&mut self, lang: Language) {
        self.executed = true;
        self.exported = false;
        let sanitized_input = self.input.replace(&[' ', '\r', '\n', '\t'][..], "");
        match self.engine().decode(sanitized_input) {
            Ok(bytes) => match String::from_utf8(bytes) {
                Ok(text) => {
                    self.output = text;
                    self.verified = true;
                    self.detail = lang.tr("解码成功", "Decode success");
                }
                Err(_) => {
                    self.output.clear();
                    self.verified = false;
                    self.detail =
                        lang.tr("解码后不是 UTF-8 文本", "Decoded bytes are not UTF-8 text");
                }
            },
            Err(err) => {
                self.output.clear();
                self.verified = false;
                self.detail = format!("{}: {err}", lang.tr("解码失败", "Decode failed"));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::Language;

    #[test]
    fn test_base64_encode() {
        let mut tool = Base64Tool {
            input: "hello".to_string(),
            ..Default::default()
        };
        tool.encode(Language::Zh);
        assert_eq!(tool.output, "aGVsbG8=");
    }

    #[test]
    fn test_base64_decode() {
        let mut tool = Base64Tool {
            input: "aGVs b G8=\r\n".to_string(), // test whitespaces
            ..Default::default()
        };
        tool.decode(Language::Zh);
        assert_eq!(tool.output, "hello");
    }
}
