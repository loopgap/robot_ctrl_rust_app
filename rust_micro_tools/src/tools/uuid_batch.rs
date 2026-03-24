use eframe::egui;
use uuid::Uuid;

use crate::guide::render_guide;
use crate::i18n::Language;
use crate::theme::ACCENT_COLOR;
use crate::workflow::{LoopState, LoopStep};

pub struct UuidBatchTool {
    count: String,
    uppercase: bool,
    no_hyphen: bool,
    output: String,
    detail: String,
    executed: bool,
    verified: bool,
    exported: bool,
}

impl Default for UuidBatchTool {
    fn default() -> Self {
        Self {
            count: "10".to_string(),
            uppercase: false,
            no_hyphen: false,
            output: String::new(),
            detail: String::new(),
            executed: false,
            verified: false,
            exported: false,
        }
    }
}

impl UuidBatchTool {
    pub fn ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, lang: Language) {
        ui.heading(lang.tr("UUID 批量生成", "UUID Batch Generator"));
        ui.label(lang.tr(
            "用于资源主键、测试数据与链路追踪标识生成。",
            "For IDs in testing, tracing and resource keys.",
        ));
        render_guide(
            ui,
            lang,
            "UUID 批量生成",
            "UUID Batch Generator",
            &[
                ("设置生成数量（1~1000）", "Set generation count (1~1000)"),
                ("选择格式（大小写/短格式）", "Choose format options"),
                ("点击生成", "Click generate"),
                ("复制结果给下游系统", "Copy output to downstream systems"),
            ],
        );
        ui.separator();

        ui.horizontal(|ui| {
            ui.label(lang.tr("数量", "Count"));
            ui.text_edit_singleline(&mut self.count);
            ui.checkbox(&mut self.uppercase, lang.tr("大写", "Uppercase"));
            ui.checkbox(&mut self.no_hyphen, lang.tr("去掉连字符", "No hyphen"));

            if ui
                .add(egui::Button::new(lang.tr("生成", "Generate")).fill(ACCENT_COLOR))
                .clicked()
            {
                self.generate(lang);
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
            [ui.available_width(), 240.0],
            egui::TextEdit::multiline(&mut self.output),
        );

        if ui.button(lang.tr("复制输出", "Copy Output")).clicked() {
            ctx.copy_text(self.output.clone());
            self.exported = !self.output.is_empty();
        }
    }

    pub fn loop_steps(&self, lang: Language) -> Vec<LoopStep> {
        vec![
            LoopStep {
                name: if matches!(lang, Language::Zh) {
                    "输入"
                } else {
                    "Input"
                },
                state: if !self.count.trim().is_empty() {
                    LoopState::Done
                } else {
                    LoopState::Pending
                },
                detail: lang.tr("输入生成数量", "Input generation count"),
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
                    lang.tr("执行后显示校验信息", "Validation info appears after run")
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
                    lang.tr("已生成 UUID", "UUID generated")
                } else {
                    lang.tr("点击生成按钮", "Click generate")
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
                    lang.tr("结果可直接用于系统配置", "Output ready for system usage")
                } else {
                    lang.tr("请检查数量范围", "Check count range")
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
                    lang.tr("复制到脚本/配置文件", "Copy to scripts/config")
                },
            },
        ]
    }

    fn generate(&mut self, lang: Language) {
        self.executed = true;
        self.exported = false;

        let count = match self.count.trim().parse::<usize>() {
            Ok(v) if (1..=1000).contains(&v) => v,
            _ => {
                self.output.clear();
                self.verified = false;
                self.detail = lang.tr("数量无效，请输入 1~1000", "Invalid count, use 1~1000");
                return;
            }
        };

        let mut lines = Vec::with_capacity(count);
        for _ in 0..count {
            let mut id = Uuid::new_v4().to_string();
            if self.no_hyphen {
                id = id.replace('-', "");
            }
            if self.uppercase {
                id = id.to_uppercase();
            }
            lines.push(id);
        }

        self.output = lines.join("\n");
        self.verified = true;
        self.detail = match lang {
            Language::Zh => format!("生成完成：{} 条 UUID", count),
            Language::En => format!("Generated: {} UUIDs", count),
        };
    }
}
