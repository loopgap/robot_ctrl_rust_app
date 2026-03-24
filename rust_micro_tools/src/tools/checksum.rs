use crc32fast::Hasher;
use eframe::egui;
use sha2::{Digest, Sha256};

use crate::guide::render_guide;
use crate::i18n::Language;
use crate::theme::ACCENT_COLOR;
use crate::workflow::{LoopState, LoopStep};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ChecksumAlgo {
    Crc32,
    Fnv1a64,
    Sha256,
}

impl ChecksumAlgo {
    pub fn label(self) -> &'static str {
        match self {
            Self::Crc32 => "CRC32",
            Self::Fnv1a64 => "FNV1a-64",
            Self::Sha256 => "SHA256",
        }
    }
}

pub struct ChecksumTool {
    input: String,
    expected: String,
    algo: ChecksumAlgo,
    output: String,
    verify_msg: String,
    executed: bool,
    exported: bool,
}

impl Default for ChecksumTool {
    fn default() -> Self {
        Self {
            input: String::new(),
            expected: String::new(),
            algo: ChecksumAlgo::Crc32,
            output: String::new(),
            verify_msg: String::new(),
            executed: false,
            exported: false,
        }
    }
}

impl ChecksumTool {
    pub fn ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, lang: Language) {
        ui.heading(lang.tr("校验和工坊", "Checksum Workshop"));
        ui.label(lang.tr(
            "用于接口联调、报文验签和数据一致性校验。",
            "For API integration and data consistency checks.",
        ));
        render_guide(
            ui,
            lang,
            "校验和",
            "Checksum",
            &[
                ("粘贴原始文本", "Paste raw payload text"),
                ("选择算法", "Choose algorithm"),
                (
                    "点击计算并比对期望值",
                    "Calculate and compare expected value",
                ),
                ("复制结果发给上下游", "Copy result to downstream systems"),
            ],
        );
        ui.separator();

        ui.label(lang.tr("输入文本", "Input Text"));
        ui.add_sized(
            [ui.available_width(), 180.0],
            egui::TextEdit::multiline(&mut self.input).hint_text("粘贴原始文本或报文内容"),
        );

        ui.horizontal(|ui| {
            egui::ComboBox::from_label(lang.tr("算法", "Algorithm"))
                .selected_text(self.algo.label())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.algo, ChecksumAlgo::Crc32, "CRC32");
                    ui.selectable_value(&mut self.algo, ChecksumAlgo::Fnv1a64, "FNV1a-64");
                    ui.selectable_value(&mut self.algo, ChecksumAlgo::Sha256, "SHA256");
                });

            if ui
                .add(egui::Button::new(lang.tr("计算", "Calculate")).fill(ACCENT_COLOR))
                .clicked()
            {
                self.execute(lang);
            }
        });

        ui.label(lang.tr("期望值（可选）", "Expected (Optional)"));
        ui.text_edit_singleline(&mut self.expected);

        ui.separator();
        ui.label(lang.tr("计算结果", "Result"));
        ui.text_edit_singleline(&mut self.output);

        if !self.verify_msg.is_empty() {
            let color = if self.verify_msg.contains("通过") {
                egui::Color32::LIGHT_GREEN
            } else {
                egui::Color32::LIGHT_RED
            };
            ui.colored_label(color, &self.verify_msg);
        }

        if ui.button(lang.tr("复制结果", "Copy Result")).clicked() {
            ctx.copy_text(self.output.clone());
            self.exported = !self.output.is_empty();
        }
    }

    pub fn loop_steps(&self, lang: Language) -> Vec<LoopStep> {
        let input_ok = !self.input.trim().is_empty();
        let verified = if self.expected.trim().is_empty() {
            LoopState::Pending
        } else if self.verify_msg.contains("通过") {
            LoopState::Done
        } else if self.executed {
            LoopState::Warning
        } else {
            LoopState::Pending
        };

        vec![
            LoopStep {
                name: if matches!(lang, Language::Zh) {
                    "输入"
                } else {
                    "Input"
                },
                state: if input_ok {
                    LoopState::Done
                } else {
                    LoopState::Pending
                },
                detail: if input_ok {
                    format!("{} bytes", self.input.len())
                } else {
                    lang.tr("等待输入文本", "Waiting for input")
                },
            },
            LoopStep {
                name: if matches!(lang, Language::Zh) {
                    "校验"
                } else {
                    "Validate"
                },
                state: if input_ok {
                    LoopState::Done
                } else {
                    LoopState::Pending
                },
                detail: format!("算法：{}", self.algo.label()),
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
                    lang.tr("已完成计算", "Calculation done")
                } else {
                    lang.tr("点击计算执行", "Click calculate")
                },
            },
            LoopStep {
                name: if matches!(lang, Language::Zh) {
                    "验证"
                } else {
                    "Verify"
                },
                state: verified,
                detail: if self.expected.trim().is_empty() {
                    lang.tr("可输入期望值进行比对", "Compare with expected")
                } else if self.verify_msg.is_empty() {
                    lang.tr("等待计算结果", "Waiting result")
                } else {
                    self.verify_msg.clone()
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
                    lang.tr("已复制到剪贴板", "Copied")
                } else {
                    lang.tr("可复制结果给上下游系统", "Copy to downstream")
                },
            },
        ]
    }

    fn execute(&mut self, lang: Language) {
        self.output = match self.algo {
            ChecksumAlgo::Crc32 => {
                let mut hasher = Hasher::new();
                hasher.update(self.input.as_bytes());
                format!("{:08X}", hasher.finalize())
            }
            ChecksumAlgo::Fnv1a64 => {
                let mut hash: u64 = 0xcbf29ce484222325;
                for byte in self.input.as_bytes() {
                    hash ^= *byte as u64;
                    hash = hash.wrapping_mul(0x100000001b3);
                }
                format!("{:016X}", hash)
            }
            ChecksumAlgo::Sha256 => {
                let mut hasher = Sha256::new();
                hasher.update(self.input.as_bytes());
                use std::fmt::Write;
                hasher
                    .finalize()
                    .iter()
                    .fold(String::with_capacity(64), |mut acc, b| {
                        let _ = write!(acc, "{:02x}", b);
                        acc
                    })
            }
        };

        self.executed = true;
        self.exported = false;
        self.verify_msg.clear();

        if !self.expected.trim().is_empty() {
            if self.output.eq_ignore_ascii_case(self.expected.trim()) {
                self.verify_msg =
                    lang.tr("校验通过：结果与期望值一致", "Verified: matched expected");
            } else {
                self.verify_msg =
                    lang.tr("校验失败：结果与期望值不一致", "Verify failed: mismatch");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::Language;

    #[test]
    fn test_checksum_sha256() {
        let mut tool = ChecksumTool {
            input: "hello world".to_string(),
            algo: ChecksumAlgo::Sha256,
            ..Default::default()
        };
        tool.execute(Language::Zh);
        assert_eq!(
            tool.output,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_checksum_crc32() {
        let mut tool = ChecksumTool {
            input: "hello world".to_string(),
            algo: ChecksumAlgo::Crc32,
            ..Default::default()
        };
        tool.execute(Language::Zh);
        assert_eq!(tool.output, "0D4A1185");
    }

    #[test]
    fn test_checksum_fnv1a() {
        let mut tool = ChecksumTool {
            input: "hello world".to_string(),
            algo: ChecksumAlgo::Fnv1a64,
            ..Default::default()
        };
        tool.execute(Language::Zh);
        assert_eq!(tool.output, "779A65E7023CD2E7");
    }
}
