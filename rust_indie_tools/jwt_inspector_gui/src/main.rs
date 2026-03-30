#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use base64::Engine as _;
use eframe::egui;
use serde_json::Value;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "JWT Inspector",
        options,
        Box::new(|_cc| Ok(Box::new(App::default()))),
    )
}

#[derive(Default)]
struct App {
    token: String,
    header_pretty: String,
    payload_pretty: String,
    status: String,
}

impl App {
    fn inspect(&mut self) {
        self.header_pretty.clear();
        self.payload_pretty.clear();

        let parts: Vec<&str> = self.token.trim().split('.').collect();
        if parts.len() < 2 {
            self.status = "Invalid JWT format. Expected header.payload.signature".to_string();
            return;
        }

        match decode_json_part(parts[0]) {
            Ok(v) => self.header_pretty = v,
            Err(e) => {
                self.status = format!("Failed to decode header: {e}");
                return;
            }
        }

        match decode_json_part(parts[1]) {
            Ok(v) => {
                self.payload_pretty = v;
                self.status = "Decoded successfully".to_string();
            }
            Err(e) => {
                self.status = format!("Failed to decode payload: {e}");
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.heading("JWT Inspector");
            ui.label("Paste a JWT token, then decode header and payload.");
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("JWT");
            ui.add_sized(
                [ui.available_width(), 90.0],
                egui::TextEdit::multiline(&mut self.token),
            );

            if ui.button("Decode").clicked() {
                self.inspect();
            }

            ui.separator();
            ui.label(format!("Status: {}", self.status));
            ui.separator();

            ui.columns(2, |cols| {
                cols[0].label("Header");
                cols[0].add_sized(
                    [cols[0].available_width(), 220.0],
                    egui::TextEdit::multiline(&mut self.header_pretty),
                );

                cols[1].label("Payload");
                cols[1].add_sized(
                    [cols[1].available_width(), 220.0],
                    egui::TextEdit::multiline(&mut self.payload_pretty),
                );
            });
        });
    }
}

fn decode_json_part(part: &str) -> Result<String, String> {
    let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(part)
        .map_err(|e| format!("base64 decode error: {e}"))?;
    let json: Value = serde_json::from_slice(&decoded).map_err(|e| format!("json error: {e}"))?;
    serde_json::to_string_pretty(&json).map_err(|e| format!("json format error: {e}"))
}
