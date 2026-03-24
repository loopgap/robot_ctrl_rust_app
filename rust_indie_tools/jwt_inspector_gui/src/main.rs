#![windows_subsystem = "windows"]

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use eframe::egui;

const ACCENT: egui::Color32 = egui::Color32::from_rgb(0, 122, 204);

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "JWT Inspector GUI",
        eframe::NativeOptions::default(),
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(App::default()))
        }),
    )
}

#[derive(Default)]
struct App {
    token: String,
    header_out: String,
    payload_out: String,
    detail: String,
    executed: bool,
    verified: bool,
    exported: bool,
}

impl App {
    fn decode(&mut self) {
        self.executed = true;
        self.exported = false;
        self.verified = false;
        self.header_out.clear();
        self.payload_out.clear();

        let parts: Vec<&str> = self.token.trim().split('.').collect();
        if parts.len() < 2 {
            self.detail = "JWT 至少需要 header.payload".to_string();
            return;
        }

        let decode_json = |part: &str| -> Result<String, String> {
            let bytes = URL_SAFE_NO_PAD
                .decode(part)
                .map_err(|e| format!("Base64URL 解码失败: {e}"))?;
            let val: serde_json::Value =
                serde_json::from_slice(&bytes).map_err(|e| format!("JSON 解析失败: {e}"))?;
            serde_json::to_string_pretty(&val).map_err(|e| format!("JSON 格式化失败: {e}"))
        };

        match (decode_json(parts[0]), decode_json(parts[1])) {
            (Ok(h), Ok(p)) => {
                self.header_out = h;
                self.payload_out = p;
                self.verified = true;
                self.detail = "解析成功（注意：本工具不校验签名）".to_string();
            }
            (Err(e), _) | (_, Err(e)) => {
                self.detail = e;
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.heading("JWT 解析工坊");
            ui.colored_label(ACCENT, "闭环流程：输入 → 校验 → 执行 → 验证 → 导出");
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.columns(2, |cols| {
                cols[0].label("JWT Token");
                cols[0].add_sized(
                    [cols[0].available_width(), 120.0],
                    egui::TextEdit::multiline(&mut self.token)
                        .hint_text("eyJ...header... . eyJ...payload... . signature"),
                );
                if cols[0]
                    .add(egui::Button::new("执行解析").fill(ACCENT))
                    .clicked()
                {
                    self.decode();
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
                cols[0].label("Header");
                cols[0].add_sized(
                    [cols[0].available_width(), 150.0],
                    egui::TextEdit::multiline(&mut self.header_out),
                );
                cols[0].label("Payload");
                cols[0].add_sized(
                    [cols[0].available_width(), 170.0],
                    egui::TextEdit::multiline(&mut self.payload_out),
                );
                if cols[0].button("复制 Payload").clicked() {
                    ctx.copy_text(self.payload_out.clone());
                    self.exported = !self.payload_out.is_empty();
                }

                cols[1].heading("流程状态");
                cols[1].separator();
                let input_ok = !self.token.trim().is_empty();
                let steps = [
                    ("输入", input_ok, "粘贴 JWT"),
                    ("校验", self.executed, "检查段数与编码"),
                    ("执行", self.executed, "解码 Header/Payload"),
                    (
                        "验证",
                        self.verified,
                        if self.detail.is_empty() {
                            "待执行"
                        } else {
                            &self.detail
                        },
                    ),
                    ("导出", self.exported, "复制结果到测试/工单"),
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
