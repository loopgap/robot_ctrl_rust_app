use crate::app::{AppState, DisplayMode, LogDirection};
use crate::i18n::Tr;
use crate::models::packet::parse_hex_string;
use crate::views::ui_kit::{page_header, settings_card};
use egui::{self, Color32, RichText, ScrollArea, TextEdit, Ui};

pub fn show(ui: &mut Ui, state: &mut AppState) {
    let theme = state.theme.clone();
    let lang = state.lang();
    page_header(ui, Tr::tab_terminal(lang), "terminal");

    // ─── 工具栏 ──────────────────────────────────────────
    settings_card(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 10.0;
            ui.label(format!("{}:", Tr::display(lang)));
            ui.selectable_value(
                &mut state.ui.display_mode,
                DisplayMode::Hex,
                Tr::display_mode_hex(lang),
            );
            ui.selectable_value(
                &mut state.ui.display_mode,
                DisplayMode::Ascii,
                Tr::display_mode_ascii(lang),
            );
            ui.selectable_value(
                &mut state.ui.display_mode,
                DisplayMode::Mixed,
                Tr::display_mode_mixed(lang),
            );

            ui.separator();
            ui.checkbox(&mut state.ui.auto_scroll, Tr::auto_scroll(lang));

            ui.separator();
            if ui.button(Tr::clear(lang)).clicked() {
                state.log.log_entries.clear();
            }

            ui.separator();
            ui.label(format!(
                "{}: {}",
                Tr::entries(lang),
                state.log.log_entries.len()
            ));
        });
    });

    ui.add_space(10.0);

    // ─── 接收区域 ────────────────────────────────────────
    let available = ui.available_height() - 140.0;
    let log_height = available.max(120.0);

    settings_card(ui, |ui| {
        ScrollArea::vertical()
            .max_height(log_height)
            .auto_shrink([false; 2])
            .stick_to_bottom(state.ui.auto_scroll)
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());

                if state.log.log_entries.is_empty() {
                    crate::views::ui_kit::empty_state(
                        ui,
                        "term",
                        Tr::no_data_yet(lang),
                        "Connect and send data to see logs.",
                        &theme,
                    );
                }

                for entry in &state.log.log_entries {
                    let (prefix, color) = match entry.direction {
                        LogDirection::Tx => ("TX", theme.status_info),
                        LogDirection::Rx => ("RX", theme.status_ok),
                        LogDirection::Info => ("INFO", theme.status_warn),
                    };

                    let formatted = format_data_with_mode(&entry.data, state.ui.display_mode);

                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing.x = 6.0;
                        ui.label(
                            RichText::new(&entry.timestamp)
                                .size(11.5)
                                .color(theme.text_muted),
                        );
                        ui.label(
                            RichText::new(format!("[{}]", entry.channel))
                                .size(11.5)
                                .color(theme.text_muted),
                        );
                        ui.label(RichText::new(prefix).size(11.5).color(color).strong());
                        ui.label(
                            RichText::new(&formatted)
                                .size(12.5)
                                .color(theme.text_primary)
                                .monospace(),
                        );
                    });
                }
            });
    });

    ui.add_space(10.0);

    // ─── 发送配置行 ──────────────────────────────────────
    settings_card(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 10.0;
            ui.checkbox(&mut state.ui.send_hex, Tr::display_mode_hex(lang));
            ui.checkbox(&mut state.ui.send_with_newline, Tr::newline(lang));

            if state.ui.send_with_newline {
                egui::ComboBox::from_id_salt("newline_type")
                    .selected_text(&state.ui.newline_type)
                    .width(70.0)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut state.ui.newline_type, "\\r\\n".into(), "\\r\\n");
                        ui.selectable_value(&mut state.ui.newline_type, "\\n".into(), "\\n");
                        ui.selectable_value(&mut state.ui.newline_type, "\\r".into(), "\\r");
                    });
            }
        });

        ui.add_space(6.0);

        // ─── 发送输入行 ──────────────────────────────────────
        ui.horizontal_wrapped(|ui| {
            let hint = if state.ui.send_hex {
                Tr::hex_hint(lang)
            } else {
                Tr::type_to_send(lang)
            };
            let response = ui.add(
                TextEdit::singleline(&mut state.ui.send_text)
                    .desired_width((ui.available_width() - 110.0).max(280.0))
                    .hint_text(hint),
            );

            let enter_pressed =
                response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

            if ui
                .button(RichText::new(Tr::send(lang)).size(13.0))
                .clicked()
                || enter_pressed
            {
                send_data(state);
            }
        });

        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            ui.label("Quick:");
            for quick in [
                state.ui.quick_cmd_1.clone(),
                state.ui.quick_cmd_2.clone(),
                state.ui.quick_cmd_3.clone(),
            ] {
                if quick.trim().is_empty() {
                    continue;
                }
                if ui.button(quick.as_str()).clicked() {
                    send_text_payload(state, &quick);
                }
            }
        });

        ui.horizontal_wrapped(|ui| {
            ui.label("Q1:");
            ui.add(TextEdit::singleline(&mut state.ui.quick_cmd_1).desired_width(120.0));
            ui.label("Q2:");
            ui.add(TextEdit::singleline(&mut state.ui.quick_cmd_2).desired_width(120.0));
            ui.label("Q3:");
            ui.add(TextEdit::singleline(&mut state.ui.quick_cmd_3).desired_width(120.0));
        });

        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            let status = state.active_status();
            let (r, g, b) = status.color_rgb();
            ui.label(
                RichText::new(format!("{}", status))
                    .size(11.5)
                    .color(Color32::from_rgb(r, g, b)),
            );
            ui.separator();
            ui.label(
                RichText::new(format!(
                    "TX: {} | RX: {} | Err: {}",
                    format_bytes_short(state.total_bytes_sent()),
                    format_bytes_short(state.total_bytes_received()),
                    state.total_errors()
                ))
                .size(11.5)
                .color(Color32::GRAY),
            );
            ui.separator();
            ui.label(
                RichText::new(format!("Link: {}", state.link_health_text()))
                    .size(11.5)
                    .color(Color32::GRAY),
            );
        });
    });
}

fn send_data(state: &mut AppState) {
    let lang = state.lang();
    let text = state.ui.send_text.clone();
    send_payload_internal(state, &text, lang);
}

fn send_text_payload(state: &mut AppState, text: &str) {
    let lang = state.lang();
    send_payload_internal(state, text, lang);
}

fn send_payload_internal(state: &mut AppState, text: &str, lang: crate::i18n::Language) {
    if text.is_empty() {
        return;
    }

    let data = if state.ui.send_hex {
        parse_hex_string(text)
    } else {
        let mut bytes = text.as_bytes().to_vec();
        if state.ui.send_with_newline {
            match state.ui.newline_type.as_str() {
                "\\r\\n" => bytes.extend_from_slice(b"\r\n"),
                "\\n" => bytes.push(b'\n'),
                "\\r" => bytes.push(b'\r'),
                _ => bytes.extend_from_slice(b"\r\n"),
            }
        }
        bytes
    };

    if data.is_empty() {
        return;
    }

    match state.send_data(&data) {
        Ok(()) => {
            state.status_message = Tr::sent_bytes(data.len(), lang);
        }
        Err(e) => {
            state.status_message = Tr::send_error(&e.to_string(), lang);
        }
    }
}

fn format_data_with_mode(data: &[u8], mode: DisplayMode) -> String {
    use std::fmt::Write;
    let mut buf = String::with_capacity(data.len() * 3);
    match mode {
        DisplayMode::Hex => {
            for (i, b) in data.iter().enumerate() {
                if i > 0 {
                    buf.push(' ');
                }
                let _ = write!(buf, "{:02X}", b);
            }
        }
        DisplayMode::Ascii => {
            buf.push_str(&String::from_utf8_lossy(data));
        }
        DisplayMode::Mixed => {
            for (i, b) in data.iter().enumerate() {
                if i > 0 {
                    buf.push(' ');
                }
                let _ = write!(buf, "{:02X}", b);
            }
            buf.push_str(" | ");
            for &b in data {
                if b.is_ascii_graphic() || b == b' ' {
                    buf.push(b as char);
                } else {
                    buf.push('.');
                }
            }
        }
    }
    buf
}

fn format_bytes_short(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{}B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1}K", bytes as f64 / 1024.0)
    } else {
        format!("{:.1}M", bytes as f64 / (1024.0 * 1024.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::DisplayMode;

    #[test]
    fn test_format_bytes_short_bytes() {
        assert_eq!(format_bytes_short(500), "500B");
    }

    #[test]
    fn test_format_bytes_short_kb() {
        assert_eq!(format_bytes_short(1024), "1.0K");
    }

    #[test]
    fn test_format_bytes_short_mb() {
        assert_eq!(format_bytes_short(1048576), "1.0M");
    }

    #[test]
    fn test_format_bytes_short_zero() {
        assert_eq!(format_bytes_short(0), "0B");
    }

    #[test]
    fn test_format_bytes_short_boundary() {
        assert_eq!(format_bytes_short(1023), "1023B");
        assert_eq!(format_bytes_short(1024), "1.0K");
        assert_eq!(format_bytes_short(1048575), "1024.0K");
        assert_eq!(format_bytes_short(1048576), "1.0M");
    }

    #[test]
    fn test_format_data_hex() {
        let result = format_data_with_mode(&[0x48, 0x65], DisplayMode::Hex);
        assert!(result.contains("48") || result.contains("65"));
    }

    #[test]
    fn test_format_data_ascii() {
        let result = format_data_with_mode(&[0x48, 0x65], DisplayMode::Ascii);
        assert!(result.contains("He"));
    }

    #[test]
    fn test_format_data_hex_empty() {
        let result = format_data_with_mode(&[], DisplayMode::Hex);
        assert!(result.is_empty() || result == "");
    }

    #[test]
    fn test_format_data_mixed() {
        let result = format_data_with_mode(&[0x48, 0x65, 0x6C], DisplayMode::Mixed);
        // Mixed mode should show both hex and ASCII
        assert!(!result.is_empty());
    }

    #[test]
    fn test_format_data_ascii_non_printable() {
        // Non-printable bytes should be handled gracefully
        let result = format_data_with_mode(&[0x00, 0x01, 0xFF], DisplayMode::Ascii);
        assert!(!result.is_empty());
    }
}
