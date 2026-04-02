use eframe::egui;

use crate::guide::render_guide;
use crate::i18n::Language;
use crate::theme::ACCENT_COLOR;
use crate::workflow::{LoopState, LoopStep};

#[derive(Default)]
pub struct CsvCleanerTool {
    input: String,
    output: String,
    dedup: bool,
    detail: String,
    executed: bool,
    verified: bool,
    exported: bool,
}

impl CsvCleanerTool {
    pub fn ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, lang: Language) {
        ui.heading(lang.tr("CSV 清洗工坊", "CSV Cleaner"));
        ui.label(lang.tr(
            "用于 CSV 列宽一致性检查、去重与快速清洗。",
            "Clean CSV rows with consistency checks and deduplication.",
        ));
        render_guide(
            ui,
            lang,
            "CSV 清洗工坊",
            "CSV Cleaner",
            &[
                ("粘贴原始 CSV", "Paste raw CSV"),
                ("按需启用去重", "Enable deduplication if needed"),
                ("执行清洗并检查结果", "Run cleanup and inspect result"),
                ("复制输出到下游系统", "Copy output to downstream systems"),
            ],
        );
        ui.separator();

        ui.label(lang.tr("输入 CSV", "Input CSV"));
        ui.add_sized(
            [ui.available_width(), 180.0],
            egui::TextEdit::multiline(&mut self.input).hint_text("col_a,col_b,..."),
        );

        ui.horizontal(|ui| {
            ui.checkbox(&mut self.dedup, lang.tr("去重相同行", "Deduplicate rows"));
            if ui
                .add(egui::Button::new(lang.tr("执行清洗", "Run cleaning")).fill(ACCENT_COLOR))
                .clicked()
            {
                self.execute(lang);
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
            egui::TextEdit::multiline(&mut self.output),
        );
        if ui.button(lang.tr("复制输出", "Copy output")).clicked() {
            ctx.copy_text(self.output.clone());
            self.exported = !self.output.trim().is_empty();
        }
    }

    pub fn loop_steps(&self, lang: Language) -> Vec<LoopStep> {
        let input_ok = !self.input.trim().is_empty();
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
                    format!("{} rows", self.input.lines().count())
                } else {
                    lang.tr("等待输入 CSV", "Waiting for CSV input")
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
                    lang.tr("待执行后生成一致性结果", "Consistency result after run")
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
                    lang.tr("已完成清洗", "Cleanup finished")
                } else {
                    lang.tr("点击执行清洗", "Click run cleaning")
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
                    lang.tr("可直接用于导入或下游处理", "Ready for import/downstream")
                } else {
                    lang.tr("执行后可验证结果", "Run to verify result")
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
                    lang.tr("已复制输出", "Copied")
                } else {
                    lang.tr("复制结果给下游系统", "Copy output to downstream")
                },
            },
        ]
    }

    fn execute(&mut self, lang: Language) {
        self.executed = true;
        self.exported = false;

        let mut rows: Vec<String> = Vec::new();
        let mut width: Option<usize> = None;
        let mut invalid = 0usize;

        for raw in self.input.lines() {
            let line = raw.trim();
            if line.is_empty() {
                continue;
            }

            let cols: Vec<String> = line.split(',').map(|c| c.trim().to_string()).collect();
            if let Some(w) = width {
                if cols.len() != w {
                    invalid += 1;
                }
            } else {
                width = Some(cols.len());
            }
            rows.push(cols.join(","));
        }

        if self.dedup {
            rows.sort();
            rows.dedup();
        }

        self.output = rows.join("\n");
        self.verified = invalid == 0;
        self.detail = if self.verified {
            match lang {
                Language::Zh => format!("清洗完成：{} 行，列宽一致", rows.len()),
                Language::En => format!("Cleaned {} rows with consistent columns", rows.len()),
            }
        } else {
            match lang {
                Language::Zh => format!("清洗完成：{} 行，{} 行列宽不一致", rows.len(), invalid),
                Language::En => format!(
                    "Cleaned {} rows, {} rows have inconsistent columns",
                    rows.len(),
                    invalid
                ),
            }
        };
    }
}
