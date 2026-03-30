#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use rust_indie_tools_core::{
    apply_dark_theme, render_action_button, render_export_button, render_header_panel,
    render_text_area, render_workflow_panel, WorkflowStep,
};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "CSV Cleaner GUI",
        options,
        Box::new(|cc| {
            apply_dark_theme(&cc.egui_ctx);
            Ok(Box::new(App::default()))
        }),
    )
}

#[derive(Default)]
struct App {
    input: String,
    output: String,
    dedup: bool,
    detail: String,
    executed: bool,
    verified: bool,
    exported: bool,
}

impl App {
    fn process(&mut self) {
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
            format!("Cleaned {} rows. Column counts are consistent.", rows.len())
        } else {
            format!(
                "Cleaned {} rows. {} rows have inconsistent columns.",
                rows.len(),
                invalid
            )
        };
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        render_header_panel(ctx, "CSV Cleaner GUI");

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.columns(2, |cols| {
                cols[0].label("Input CSV");
                render_text_area(&mut cols[0], &mut self.input, 220.0);
                cols[0].checkbox(&mut self.dedup, "Deduplicate rows");
                if render_action_button(&mut cols[0], "Run cleaning").clicked() {
                    self.process();
                }

                cols[0].separator();
                cols[0].label("Output");
                render_text_area(&mut cols[0], &mut self.output, 220.0);
                if render_export_button(&mut cols[0], "Copy output").clicked() {
                    ctx.copy_text(self.output.clone());
                    self.exported = !self.output.is_empty();
                }

                let input_ok = !self.input.trim().is_empty();
                let steps = [
                    WorkflowStep {
                        name: "Input",
                        done: input_ok,
                        detail: "Paste or type original CSV content",
                    },
                    WorkflowStep {
                        name: "Validate",
                        done: self.executed,
                        detail: "Validate that row widths are consistent",
                    },
                    WorkflowStep {
                        name: "Execute",
                        done: self.executed,
                        detail: "Apply trim and optional deduplication",
                    },
                    WorkflowStep {
                        name: "Verify",
                        done: self.executed && self.verified,
                        detail: if self.detail.is_empty() {
                            "Not executed"
                        } else {
                            &self.detail
                        },
                    },
                    WorkflowStep {
                        name: "Export",
                        done: self.exported,
                        detail: "Copy cleaned result to clipboard",
                    },
                ];
                render_workflow_panel(&mut cols[1], &steps);
            });
        });
    }
}
