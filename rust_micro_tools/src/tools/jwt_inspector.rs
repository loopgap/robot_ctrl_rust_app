use base64::Engine as _;
use eframe::egui;
use serde_json::Value;

use crate::guide::render_guide;
use crate::i18n::Language;
use crate::theme::ACCENT_COLOR;
use crate::workflow::{LoopState, LoopStep};

#[derive(Default)]
pub struct JwtInspectorTool {
    token: String,
    header_pretty: String,
    payload_pretty: String,
    status: String,
    executed: bool,
    verified: bool,
    exported: bool,
}

impl JwtInspectorTool {
    pub fn ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, lang: Language) {
        ui.heading(lang.tr("JWT 解析工坊", "JWT Inspector"));
        ui.label(lang.tr(
            "用于快速查看 JWT Header/Payload（不做签名验签）。",
            "Inspect JWT header/payload quickly (without signature verification).",
        ));
        render_guide(
            ui,
            lang,
            "JWT 解析工坊",
            "JWT Inspector",
            &[
                ("粘贴 JWT 字符串", "Paste JWT token"),
                ("执行解析", "Decode token"),
                ("检查 Header 与 Payload", "Inspect header and payload"),
                ("复制解析结果", "Copy decoded result"),
            ],
        );
        ui.separator();

        ui.label(lang.tr("JWT", "JWT"));
        ui.add_sized(
            [ui.available_width(), 90.0],
            egui::TextEdit::multiline(&mut self.token),
        );

        if ui
            .add(egui::Button::new(lang.tr("解析", "Decode")).fill(ACCENT_COLOR))
            .clicked()
        {
            self.inspect(lang);
        }

        if !self.status.is_empty() {
            let color = if self.verified {
                egui::Color32::LIGHT_GREEN
            } else {
                egui::Color32::LIGHT_RED
            };
            ui.colored_label(color, &self.status);
        }

        ui.separator();
        ui.columns(2, |cols| {
            cols[0].label(lang.tr("Header", "Header"));
            cols[0].add_sized(
                [cols[0].available_width(), 200.0],
                egui::TextEdit::multiline(&mut self.header_pretty),
            );

            cols[1].label(lang.tr("Payload", "Payload"));
            cols[1].add_sized(
                [cols[1].available_width(), 200.0],
                egui::TextEdit::multiline(&mut self.payload_pretty),
            );
        });

        if ui
            .button(lang.tr("复制 Header+Payload", "Copy Header+Payload"))
            .clicked()
        {
            let mut text = String::new();
            if !self.header_pretty.trim().is_empty() {
                text.push_str("# Header\n");
                text.push_str(&self.header_pretty);
                text.push_str("\n\n");
            }
            if !self.payload_pretty.trim().is_empty() {
                text.push_str("# Payload\n");
                text.push_str(&self.payload_pretty);
            }
            ctx.copy_text(text.clone());
            self.exported = !text.trim().is_empty();
        }
    }

    pub fn loop_steps(&self, lang: Language) -> Vec<LoopStep> {
        let has_input = !self.token.trim().is_empty();
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
                    lang.tr("已输入 JWT", "JWT provided")
                } else {
                    lang.tr("等待 JWT 输入", "Waiting for JWT input")
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
                detail: if self.status.is_empty() {
                    lang.tr("待执行后校验格式", "Validate format after decode")
                } else {
                    self.status.clone()
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
                    lang.tr("已完成解析", "Decode executed")
                } else {
                    lang.tr("点击解析", "Click decode")
                },
            },
            LoopStep {
                name: if matches!(lang, Language::Zh) {
                    "验证"
                } else {
                    "Verify"
                },
                state: if self.executed && self.verified {
                    LoopState::Done
                } else if self.executed {
                    LoopState::Warning
                } else {
                    LoopState::Pending
                },
                detail: if self.verified {
                    lang.tr("Header/Payload 解析成功", "Header/Payload decoded")
                } else {
                    lang.tr("请检查 JWT 结构或编码", "Check JWT structure/encoding")
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
                    lang.tr("已复制解析结果", "Copied decoded result")
                } else {
                    lang.tr("可复制解析结果", "Copy decoded result")
                },
            },
        ]
    }

    fn inspect(&mut self, lang: Language) {
        self.executed = true;
        self.exported = false;
        self.header_pretty.clear();
        self.payload_pretty.clear();

        let parts: Vec<&str> = self.token.trim().split('.').collect();
        if parts.len() < 2 {
            self.verified = false;
            self.status = lang.tr(
                "JWT 格式错误，应为 header.payload.signature",
                "Invalid JWT format: expected header.payload.signature",
            );
            return;
        }

        match decode_json_part(parts[0]) {
            Ok(v) => self.header_pretty = v,
            Err(e) => {
                self.verified = false;
                self.status = match lang {
                    Language::Zh => format!("Header 解析失败: {}", e),
                    Language::En => format!("Failed to decode header: {}", e),
                };
                return;
            }
        }

        match decode_json_part(parts[1]) {
            Ok(v) => {
                self.payload_pretty = v;
                self.verified = true;
                self.status = lang.tr("解析成功", "Decoded successfully");
            }
            Err(e) => {
                self.verified = false;
                self.status = match lang {
                    Language::Zh => format!("Payload 解析失败: {}", e),
                    Language::En => format!("Failed to decode payload: {}", e),
                };
            }
        }
    }
}

fn decode_json_part(part: &str) -> Result<String, String> {
    let normalized = part.trim();
    let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(normalized)
        .or_else(|_| base64::engine::general_purpose::URL_SAFE.decode(normalized))
        .map_err(|e| format!("base64 decode error: {}", e))?;

    let json: Value = serde_json::from_slice(&decoded).map_err(|e| format!("json error: {}", e))?;
    serde_json::to_string_pretty(&json).map_err(|e| format!("json format error: {}", e))
}
