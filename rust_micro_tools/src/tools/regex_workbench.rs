use eframe::egui;
use regex::Regex;

use crate::guide::render_guide;
use crate::i18n::Language;
use crate::theme::ACCENT_COLOR;
use crate::workflow::{LoopState, LoopStep};

#[derive(Default)]
pub struct RegexWorkbenchTool {
    pattern: String,
    input: String,
    output: String,
    matched: usize,
    detail: String,
    executed: bool,
    verified: bool,
    exported: bool,
}

impl RegexWorkbenchTool {
    pub fn ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, lang: Language) {
        ui.heading(lang.tr("Regex 巡检工坊", "Regex Workbench"));
        ui.label(lang.tr(
            "用于日志筛选、文本巡检与规则验证。",
            "For log filtering, text inspection, and pattern verification.",
        ));
        render_guide(
            ui,
            lang,
            "Regex 巡检工坊",
            "Regex Workbench",
            &[
                ("输入正则表达式", "Enter regex pattern"),
                ("粘贴待匹配文本", "Paste input text"),
                ("执行匹配并检查命中", "Run matching and inspect hits"),
                ("复制命中结果", "Copy matched output"),
            ],
        );
        ui.separator();

        ui.label(lang.tr("正则表达式", "Regex Pattern"));
        ui.text_edit_singleline(&mut self.pattern);

        ui.label(lang.tr("输入文本", "Input Text"));
        ui.add_sized(
            [ui.available_width(), 180.0],
            egui::TextEdit::multiline(&mut self.input),
        );

        if ui
            .add(egui::Button::new(lang.tr("执行匹配", "Run Match")).fill(ACCENT_COLOR))
            .clicked()
        {
            self.run(lang);
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
            Language::Zh => format!("命中结果（{} 行）", self.matched),
            Language::En => format!("Matched result ({} lines)", self.matched),
        });
        ui.add_sized(
            [ui.available_width(), 180.0],
            egui::TextEdit::multiline(&mut self.output),
        );

        if ui
            .button(lang.tr("复制命中结果", "Copy Matched Output"))
            .clicked()
        {
            ctx.copy_text(self.output.clone());
            self.exported = !self.output.trim().is_empty();
        }
    }

    pub fn loop_steps(&self, lang: Language) -> Vec<LoopStep> {
        let input_ok = !self.input.trim().is_empty() && !self.pattern.trim().is_empty();
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
                    lang.tr("规则与文本已就绪", "Pattern and input ready")
                } else {
                    lang.tr("请输入规则与文本", "Provide pattern and input")
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
                    lang.tr("待执行后校验正则", "Regex validation after run")
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
                    lang.tr("已执行匹配", "Matching executed")
                } else {
                    lang.tr("点击执行匹配", "Click run match")
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
                detail: if self.executed {
                    match lang {
                        Language::Zh => format!("命中 {} 行", self.matched),
                        Language::En => format!("{} lines matched", self.matched),
                    }
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
                    lang.tr("已复制命中结果", "Copied matched output")
                } else {
                    lang.tr("可复制命中结果", "Copy matched output")
                },
            },
        ]
    }

    fn run(&mut self, lang: Language) {
        self.executed = true;
        self.exported = false;
        self.output.clear();
        self.matched = 0;

        let reg = match Regex::new(self.pattern.trim()) {
            Ok(r) => r,
            Err(e) => {
                self.verified = false;
                self.detail = match lang {
                    Language::Zh => format!("正则错误: {}", e),
                    Language::En => format!("Regex error: {}", e),
                };
                return;
            }
        };

        let mut hits = Vec::new();
        for line in self.input.lines() {
            if reg.is_match(line) {
                hits.push(line.to_string());
                self.matched += 1;
            }
        }

        self.output = hits.join("\n");
        self.verified = true;
        self.detail = match lang {
            Language::Zh => format!("执行完成：命中 {} 行", self.matched),
            Language::En => format!("Done: {} lines matched", self.matched),
        };
    }
}
