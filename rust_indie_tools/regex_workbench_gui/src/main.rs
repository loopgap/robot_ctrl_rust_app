#![windows_subsystem = "windows"]

use eframe::egui;
use regex::Regex;

const ACCENT: egui::Color32 = egui::Color32::from_rgb(0, 122, 204);

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "Regex Workbench GUI",
        eframe::NativeOptions::default(),
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(App::default()))
        }),
    )
}

#[derive(Default)]
struct App {
    pattern: String,
    input: String,
    output: String,
    matched: usize,
    detail: String,
    executed: bool,
    verified: bool,
    exported: bool,
}

impl App {
    fn run(&mut self) {
        self.executed = true;
        self.exported = false;
        self.output.clear();
        self.matched = 0;

        let reg = match Regex::new(self.pattern.trim()) {
            Ok(r) => r,
            Err(e) => {
                self.verified = false;
                self.detail = format!("正则错误: {e}");
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
        self.detail = format!("执行完成：命中 {} 行", self.matched);
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.heading("Regex 巡检工坊");
            ui.colored_label(ACCENT, "闭环流程：输入 → 校验 → 执行 → 验证 → 导出");
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.columns(2, |cols| {
                cols[0].label("正则表达式");
                cols[0].text_edit_singleline(&mut self.pattern);
                cols[0].label("输入文本");
                cols[0].add_sized(
                    [cols[0].available_width(), 220.0],
                    egui::TextEdit::multiline(&mut self.input),
                );
                if cols[0]
                    .add(egui::Button::new("执行匹配").fill(ACCENT))
                    .clicked()
                {
                    self.run();
                }
                if !self.detail.is_empty() {
                    let color = if self.verified {
                        egui::Color32::LIGHT_GREEN
                    } else {
                        egui::Color32::LIGHT_RED
                    };
                    cols[0].colored_label(color, &self.detail);
                }
                cols[0].separator();
                cols[0].label(format!("命中结果（{} 行）", self.matched));
                cols[0].add_sized(
                    [cols[0].available_width(), 220.0],
                    egui::TextEdit::multiline(&mut self.output),
                );
                if cols[0].button("复制命中结果").clicked() {
                    ctx.copy_text(self.output.clone());
                    self.exported = !self.output.is_empty();
                }

                cols[1].heading("流程状态");
                cols[1].separator();
                let input_ok = !self.input.trim().is_empty() && !self.pattern.trim().is_empty();
                let steps = [
                    ("输入", input_ok, "输入规则与文本"),
                    ("校验", self.executed, "校验正则语法"),
                    ("执行", self.executed, "逐行匹配"),
                    (
                        "验证",
                        self.executed && self.verified,
                        if self.detail.is_empty() {
                            "待执行"
                        } else {
                            &self.detail
                        },
                    ),
                    ("导出", self.exported, "复制结果到告警/工单"),
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
