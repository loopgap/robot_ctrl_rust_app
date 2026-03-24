#![windows_subsystem = "windows"]

use eframe::egui;

const ACCENT: egui::Color32 = egui::Color32::from_rgb(0, 122, 204);

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "CSV Cleaner GUI",
        options,
        Box::new(|cc| {
            let mut visuals = egui::Visuals::dark();
            visuals.override_text_color = Some(egui::Color32::from_rgb(220, 220, 230));
            cc.egui_ctx.set_visuals(visuals);
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
        let mut rows = Vec::new();
        let mut width = None;
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
            format!("清洗完成：{} 行，列数一致", rows.len())
        } else {
            format!("清洗完成：{} 行，{} 行列数异常", rows.len(), invalid)
        };
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.heading("CSV 清洗工坊");
            ui.colored_label(ACCENT, "闭环流程：输入 → 校验 → 执行 → 验证 → 导出");
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.columns(2, |cols| {
                cols[0].label("输入 CSV");
                cols[0].add_sized(
                    [cols[0].available_width(), 220.0],
                    egui::TextEdit::multiline(&mut self.input).hint_text("a,b,c"),
                );
                cols[0].checkbox(&mut self.dedup, "按行去重");
                if cols[0]
                    .add(egui::Button::new("执行清洗").fill(ACCENT))
                    .clicked()
                {
                    self.process();
                }
                cols[0].separator();
                cols[0].label("输出");
                cols[0].add_sized(
                    [cols[0].available_width(), 220.0],
                    egui::TextEdit::multiline(&mut self.output),
                );
                if cols[0].button("复制输出").clicked() {
                    ctx.copy_text(self.output.clone());
                    self.exported = !self.output.is_empty();
                }

                cols[1].heading("流程状态");
                cols[1].separator();
                let input_ok = !self.input.trim().is_empty();
                let steps = [
                    ("输入", input_ok, "粘贴 CSV 原始数据"),
                    ("校验", self.executed, "检查列数是否一致"),
                    ("执行", self.executed, "执行清洗策略"),
                    (
                        "验证",
                        self.executed && self.verified,
                        if self.detail.is_empty() {
                            "待执行"
                        } else {
                            &self.detail
                        },
                    ),
                    ("导出", self.exported, "复制给下游系统"),
                ];

                for (name, done, detail) in steps {
                    let icon = if done { "●" } else { "○" };
                    let color = if done {
                        egui::Color32::LIGHT_GREEN
                    } else {
                        egui::Color32::GRAY
                    };
                    egui::Frame::group(cols[1].style()).show(&mut cols[1], |ui| {
                        ui.colored_label(color, icon);
                        ui.strong(name);
                        ui.label(detail);
                    });
                    cols[1].add_space(4.0);
                }
            });
        });
    }
}
