use eframe::egui;

use crate::guide::render_guide;
use crate::i18n::Language;
use crate::theme::ACCENT_COLOR;
use crate::workflow::{LoopState, LoopStep};

#[derive(Default)]
pub struct UrlCodecTool {
    input: String,
    output: String,
    detail: String,
    executed: bool,
    verified: bool,
    exported: bool,
}

impl UrlCodecTool {
    pub fn ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, lang: Language) {
        ui.heading(lang.tr("URL 编解码", "URL Codec"));
        ui.label(lang.tr(
            "用于参数透传与回调链接调试。",
            "For URL parameter and callback debugging.",
        ));
        render_guide(
            ui,
            lang,
            "URL 编解码",
            "URL Codec",
            &[
                ("输入 URL 或参数串", "Input URL or query string"),
                ("选择 Encode 或 Decode", "Choose encode/decode"),
                ("查看错误提示并修正格式", "Check errors and fix format"),
                ("复制结果用于联调", "Copy output for integration"),
            ],
        );
        ui.separator();

        ui.label(lang.tr("输入内容", "Input"));
        ui.add_sized(
            [ui.available_width(), 180.0],
            egui::TextEdit::multiline(&mut self.input).hint_text("输入 URL 或参数字符串"),
        );

        ui.horizontal(|ui| {
            if ui
                .add(egui::Button::new("URL Encode").fill(ACCENT_COLOR))
                .clicked()
            {
                self.encode(lang);
            }
            if ui.button("URL Decode").clicked() {
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
            [ui.available_width(), 180.0],
            egui::TextEdit::multiline(&mut self.output).hint_text("编码/解码结果"),
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
                    lang.tr("等待输入 URL/参数", "Waiting for input")
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
                    lang.tr("执行后生成编码校验信息", "Validation after run")
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
                    lang.tr("已完成 Encode/Decode", "Operation done")
                } else {
                    lang.tr("点击 Encode 或 Decode", "Click action")
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
                    lang.tr("结果可直接用于联调", "Ready for integration")
                } else {
                    lang.tr("输入可能不是合法编码串", "Invalid encoded input")
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
                    lang.tr("复制结果到接口文档/工单", "Copy to docs/tickets")
                },
            },
        ]
    }

    fn encode(&mut self, lang: Language) {
        self.output = urlencoding::encode(&self.input).into_owned();
        self.executed = true;
        self.verified = true;
        self.exported = false;
        self.detail = lang.tr("编码成功", "Encode success");
    }

    fn decode(&mut self, lang: Language) {
        self.executed = true;
        self.exported = false;
        match urlencoding::decode(&self.input) {
            Ok(value) => {
                self.output = value.into_owned();
                self.verified = true;
                self.detail = lang.tr("解码成功", "Decode success");
            }
            Err(err) => {
                self.output.clear();
                self.verified = false;
                self.detail = format!("{}: {err}", lang.tr("解码失败", "Decode failed"));
            }
        }
    }
}
