use base64::Engine as _;
use eframe::egui;
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde_json::Value;

use crate::file_ops::open_text_file;
use crate::guide::render_guide;
use crate::i18n::Language;
use crate::theme::ACCENT_COLOR;
use crate::workflow::{LoopState, LoopStep};

#[derive(Clone, Copy, PartialEq, Eq)]
enum VerifyMode {
    Auto,
    Hs256,
    Rs256,
}

impl VerifyMode {
    fn label(self) -> &'static str {
        match self {
            Self::Auto => "Auto",
            Self::Hs256 => "HS256",
            Self::Rs256 => "RS256",
        }
    }

    fn algorithm_from_header(self, token: &str) -> Result<Algorithm, String> {
        if !matches!(self, Self::Auto) {
            return Ok(match self {
                Self::Auto => unreachable!(),
                Self::Hs256 => Algorithm::HS256,
                Self::Rs256 => Algorithm::RS256,
            });
        }

        let header = decode_header(token).map_err(|e| format!("header decode error: {}", e))?;
        match header.alg {
            Algorithm::HS256 => Ok(Algorithm::HS256),
            Algorithm::RS256 => Ok(Algorithm::RS256),
            other => Err(format!("unsupported algorithm: {:?}", other)),
        }
    }
}

pub struct JwtInspectorTool {
    token: String,
    header_pretty: String,
    payload_pretty: String,
    verification_key: String,
    status: String,
    verify_mode: VerifyMode,
    executed: bool,
    verified: bool,
    exported: bool,
}

impl Default for JwtInspectorTool {
    fn default() -> Self {
        Self {
            token: String::new(),
            header_pretty: String::new(),
            payload_pretty: String::new(),
            verification_key: String::new(),
            status: String::new(),
            verify_mode: VerifyMode::Auto,
            executed: false,
            verified: false,
            exported: false,
        }
    }
}

impl JwtInspectorTool {
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn load_input(&mut self, text: String) {
        self.token = text;
        self.header_pretty.clear();
        self.payload_pretty.clear();
        self.status.clear();
        self.executed = false;
        self.verified = false;
        self.exported = false;
    }

    pub fn output_text(&self) -> Option<String> {
        let mut text = String::new();
        if !self.header_pretty.trim().is_empty() {
            text.push_str("# Header\n");
            text.push_str(&self.header_pretty);
            text.push_str("\n\n");
        }
        if !self.payload_pretty.trim().is_empty() {
            text.push_str("# Payload\n");
            text.push_str(&self.payload_pretty);
            text.push_str("\n\n");
        }
        if !self.status.trim().is_empty() {
            text.push_str("# Status\n");
            text.push_str(&self.status);
        }

        if text.trim().is_empty() {
            None
        } else {
            Some(text)
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, lang: Language) {
        let two_columns = ui.available_width() >= 900.0;
        let token_height = if two_columns { 120.0 } else { 104.0 };
        let key_height = if two_columns { 112.0 } else { 92.0 };
        let output_height = if two_columns { 260.0 } else { 220.0 };

        ui.heading(lang.tr("JWT 解析工坊", "JWT Inspector"));
        ui.label(lang.tr(
            "用于快速解析 JWT，并支持可选的 HS256 / RS256 验签。",
            "Inspect JWT quickly and optionally verify signatures with HS256 / RS256.",
        ));
        render_guide(
            ui,
            lang,
            "JWT 解析工坊",
            "JWT Inspector",
            &[
                ("粘贴 JWT 字符串", "Paste JWT token"),
                (
                    "先执行解析，再按需验签",
                    "Decode first, then verify if needed",
                ),
                ("可粘贴或导入 secret / PEM", "Paste or import secret / PEM"),
                (
                    "复制结果用于联调记录",
                    "Copy the result for debugging notes",
                ),
            ],
        );
        ui.separator();

        ui.label(lang.tr("JWT", "JWT"));
        ui.add_sized(
            [ui.available_width(), token_height],
            egui::TextEdit::multiline(&mut self.token).hint_text("header.payload.signature"),
        );

        ui.horizontal_wrapped(|ui| {
            if ui
                .add(egui::Button::new(lang.tr("解析", "Decode")).fill(ACCENT_COLOR))
                .clicked()
            {
                self.inspect(lang);
            }

            if ui.button(lang.tr("仅清空结果", "Clear Results")).clicked() {
                self.header_pretty.clear();
                self.payload_pretty.clear();
                self.status.clear();
                self.executed = false;
                self.verified = false;
                self.exported = false;
            }
        });

        ui.separator();
        ui.label(lang.tr("可选验签", "Optional Verification"));
        ui.horizontal_wrapped(|ui| {
            egui::ComboBox::from_label(lang.tr("算法", "Algorithm"))
                .selected_text(self.verify_mode.label())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.verify_mode, VerifyMode::Auto, "Auto");
                    ui.selectable_value(&mut self.verify_mode, VerifyMode::Hs256, "HS256");
                    ui.selectable_value(&mut self.verify_mode, VerifyMode::Rs256, "RS256");
                });

            if ui
                .button(lang.tr("导入密钥文件", "Import Key File"))
                .clicked()
            {
                match open_text_file() {
                    Ok(Some((path, text))) => {
                        self.verification_key = text;
                        self.status = match lang {
                            Language::Zh => format!("已导入密钥文件：{}", path.display()),
                            Language::En => format!("Imported key file: {}", path.display()),
                        };
                    }
                    Ok(None) => {}
                    Err(err) => {
                        self.status = err;
                        self.verified = false;
                    }
                }
            }

            if ui
                .add(egui::Button::new(lang.tr("执行验签", "Verify")).fill(ACCENT_COLOR))
                .clicked()
            {
                self.verify_signature(lang);
            }
        });
        ui.add_sized(
            [ui.available_width(), key_height],
            egui::TextEdit::multiline(&mut self.verification_key).hint_text(lang.tr(
                "HS256 输入 secret；RS256 输入 PEM 公钥/证书",
                "Use secret for HS256; use public key / certificate PEM for RS256",
            )),
        );

        if !self.status.is_empty() {
            let color = if self.verified {
                egui::Color32::LIGHT_GREEN
            } else if self.executed {
                egui::Color32::LIGHT_RED
            } else {
                egui::Color32::GRAY
            };
            ui.colored_label(color, &self.status);
        }

        ui.separator();
        if two_columns {
            ui.columns(2, |cols| {
                self.render_output_panel(&mut cols[0], lang, true, output_height);
                self.render_output_panel(&mut cols[1], lang, false, output_height);
            });
        } else {
            self.render_output_panel(ui, lang, true, output_height);
            ui.separator();
            self.render_output_panel(ui, lang, false, output_height);
        }

        if ui
            .button(lang.tr(
                "复制 Header + Payload + 状态",
                "Copy Header + Payload + Status",
            ))
            .clicked()
        {
            if let Some(text) = self.output_text() {
                ctx.copy_text(text);
                self.exported = true;
            } else {
                self.exported = false;
            }
        }
    }

    fn render_output_panel(
        &mut self,
        ui: &mut egui::Ui,
        lang: Language,
        header: bool,
        output_height: f32,
    ) {
        if header {
            ui.label(lang.tr("头部 Header", "Header"));
            ui.add_sized(
                [ui.available_width(), output_height],
                egui::TextEdit::multiline(&mut self.header_pretty),
            );
        } else {
            ui.label(lang.tr("载荷 Payload", "Payload"));
            ui.add_sized(
                [ui.available_width(), output_height],
                egui::TextEdit::multiline(&mut self.payload_pretty),
            );
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
                    lang.tr(
                        "待解析或验签后生成状态",
                        "State appears after decode/verify",
                    )
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
                    lang.tr("已完成解析或验签", "Decode or verify executed")
                } else {
                    lang.tr("点击解析或验签", "Click decode or verify")
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
                    lang.tr(
                        "解析成功或签名验证通过",
                        "Decoded successfully or signature verified",
                    )
                } else {
                    lang.tr("请检查 token、算法或密钥", "Check token, algorithm, or key")
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
                    lang.tr("可复制结果", "Copy the result")
                },
            },
        ]
    }

    fn inspect(&mut self, lang: Language) {
        self.executed = true;
        self.exported = false;
        self.verified = false;
        self.header_pretty.clear();
        self.payload_pretty.clear();

        let parts: Vec<&str> = self.token.trim().split('.').collect();
        if parts.len() < 2 {
            self.status = lang.tr(
                "JWT 格式错误，应为 header.payload.signature",
                "Invalid JWT format: expected header.payload.signature",
            );
            return;
        }

        match decode_json_part(parts[0]) {
            Ok(v) => self.header_pretty = v,
            Err(e) => {
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
                self.status = match lang {
                    Language::Zh => format!("Payload 解析失败: {}", e),
                    Language::En => format!("Failed to decode payload: {}", e),
                };
            }
        }
    }

    fn verify_signature(&mut self, lang: Language) {
        self.executed = true;
        self.exported = false;

        if self.token.trim().is_empty() {
            self.verified = false;
            self.status = lang.tr("请先输入 JWT", "Provide a JWT first");
            return;
        }

        if self.verification_key.trim().is_empty() {
            self.verified = false;
            self.status = lang.tr("请先输入或导入验签密钥", "Provide a verification key first");
            return;
        }

        let algorithm = match self.verify_mode.algorithm_from_header(self.token.trim()) {
            Ok(algorithm) => algorithm,
            Err(err) => {
                self.verified = false;
                self.status = match lang {
                    Language::Zh => format!("无法确定验签算法: {}", err),
                    Language::En => format!("Unable to determine algorithm: {}", err),
                };
                return;
            }
        };

        let decoding_key = match algorithm {
            Algorithm::HS256 => DecodingKey::from_secret(self.verification_key.trim().as_bytes()),
            Algorithm::RS256 => match DecodingKey::from_rsa_pem(self.verification_key.as_bytes()) {
                Ok(key) => key,
                Err(err) => {
                    self.verified = false;
                    self.status = match lang {
                        Language::Zh => format!("RSA 密钥解析失败: {}", err),
                        Language::En => format!("Failed to parse RSA key: {}", err),
                    };
                    return;
                }
            },
            _ => unreachable!(),
        };

        let mut validation = Validation::new(algorithm);
        validation.validate_exp = false;
        validation.validate_nbf = false;
        validation.required_spec_claims.clear();

        match decode::<Value>(self.token.trim(), &decoding_key, &validation) {
            Ok(data) => {
                self.verified = true;
                self.payload_pretty = serde_json::to_string_pretty(&data.claims)
                    .unwrap_or_else(|_| self.payload_pretty.clone());
                self.status = match lang {
                    Language::Zh => format!("验签通过：{:?}", algorithm),
                    Language::En => format!("Signature verified: {:?}", algorithm),
                };
            }
            Err(err) => {
                self.verified = false;
                self.status = match lang {
                    Language::Zh => format!("验签失败: {}", err),
                    Language::En => format!("Verification failed: {}", err),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::Language;
    use jsonwebtoken::{encode, EncodingKey, Header};
    use rsa::pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding};
    use rsa::rand_core::OsRng;
    use rsa::RsaPrivateKey;

    #[test]
    fn test_decode_json_part_handles_json() {
        let part = "eyJzdWIiOiJkZW1vIn0";
        let decoded = decode_json_part(part).expect("decoded");
        assert!(decoded.contains("\"sub\": \"demo\""));
    }

    #[test]
    fn test_verify_hs256_success() {
        let token = encode(
            &Header::new(Algorithm::HS256),
            &serde_json::json!({"sub":"demo"}),
            &EncodingKey::from_secret(b"secret-123"),
        )
        .expect("token");

        let mut tool = JwtInspectorTool {
            token,
            verification_key: "secret-123".to_string(),
            verify_mode: VerifyMode::Hs256,
            ..Default::default()
        };

        tool.verify_signature(Language::En);
        assert!(tool.verified);
        assert!(tool.status.contains("verified"));
    }

    #[test]
    fn test_verify_rs256_success() {
        let private = RsaPrivateKey::new(&mut OsRng, 2048).expect("private key");
        let public_pem = private
            .to_public_key()
            .to_public_key_pem(LineEnding::LF)
            .expect("public pem");
        let private_pem = private
            .to_pkcs8_pem(LineEnding::LF)
            .expect("private pem")
            .to_string();

        let token = encode(
            &Header::new(Algorithm::RS256),
            &serde_json::json!({"sub":"robot","scope":"read"}),
            &EncodingKey::from_rsa_pem(private_pem.as_bytes()).expect("encoding key"),
        )
        .expect("token");

        let mut tool = JwtInspectorTool {
            token,
            verification_key: public_pem,
            verify_mode: VerifyMode::Rs256,
            ..Default::default()
        };

        tool.verify_signature(Language::En);
        assert!(tool.verified);
        assert!(tool.status.contains("RS256"));
    }
}
