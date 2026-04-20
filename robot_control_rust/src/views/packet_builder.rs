use crate::app::AppState;
use crate::i18n::Tr;
use crate::models::packet::*;
use crate::models::{Endianness, FieldType};
use crate::views::ui_kit::{page_header, settings_card};
use egui::{self, Color32, RichText, ScrollArea, Ui};

pub fn show(ui: &mut Ui, state: &mut AppState) {
    let lang = state.lang();
    page_header(ui, Tr::tab_packet_builder(lang), "packet");

    // ─── Builder / Parser 双标签 ─────────────────────────
    settings_card(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            let tab_labels = [
                (0, Tr::builder_tab(lang).to_string()),
                (1, Tr::parser_tab(lang).to_string()),
            ];
            for (idx, label) in &tab_labels {
                let selected = state.ui.packet_builder_tab == *idx;
                let btn = egui::Button::new(RichText::new(label).size(14.0).color(if selected {
                    Color32::WHITE
                } else {
                    Color32::GRAY
                }))
                .fill(if selected {
                    Color32::from_rgb(50, 60, 90)
                } else {
                    Color32::TRANSPARENT
                })
                .corner_radius(egui::CornerRadius {
                    nw: 6,
                    ne: 6,
                    sw: 0,
                    se: 0,
                })
                .min_size(egui::vec2(120.0, 30.0));
                if ui.add(btn).clicked() {
                    state.ui.packet_builder_tab = *idx;
                }
            }
        });
    });

    ui.add_space(10.0);

    match state.ui.packet_builder_tab {
        0 => show_builder(ui, state),
        _ => show_parser(ui, state),
    }
}

// ═══════════════════════════════════════════════════════════════
// Builder 标签页
// ═══════════════════════════════════════════════════════════════

fn show_builder(ui: &mut Ui, state: &mut AppState) {
    let lang = state.lang();

    // ─── 模板选择 ────────────────────────────────────────
    settings_card(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 10.0;
            ui.label(RichText::new(format!("{}:", Tr::template(lang))).strong());
            let names: Vec<String> = state
                .packet_templates
                .iter()
                .map(|t| t.name.clone())
                .collect();
            egui::ComboBox::from_id_salt("pkt_template")
                .selected_text(if state.ui.packet_template_idx < names.len() {
                    &names[state.ui.packet_template_idx]
                } else {
                    Tr::select(lang)
                })
                .width(220.0)
                .show_ui(ui, |ui| {
                    for (i, name) in names.iter().enumerate() {
                        ui.selectable_value(&mut state.ui.packet_template_idx, i, name);
                    }
                });

            if ui.button(Tr::new_template(lang)).clicked() {
                state.packet_templates.push(PacketTemplate::default());
                state.ui.packet_template_idx = state.packet_templates.len() - 1;
            }

            if state.packet_templates.len() > 1
                && ui.button(Tr::delete(lang)).clicked()
                && state.ui.packet_template_idx < state.packet_templates.len()
            {
                state.packet_templates.remove(state.ui.packet_template_idx);
                if state.ui.packet_template_idx >= state.packet_templates.len() {
                    state.ui.packet_template_idx = state.packet_templates.len().saturating_sub(1);
                }
            }
        });
    });

    let idx = state.ui.packet_template_idx;
    if idx >= state.packet_templates.len() {
        return;
    }

    ui.add_space(10.0);

    // ─── 模板配置 ────────────────────────────────────────
    settings_card(ui, |ui| {
        egui::Grid::new("pkt_config_grid")
            .num_columns(4)
            .spacing([16.0, 8.0])
            .show(ui, |ui| {
                ui.label(format!("{}:", Tr::name(lang)));
                ui.text_edit_singleline(&mut state.packet_templates[idx].name);
                ui.label(format!("{}:", Tr::description(lang)));
                ui.text_edit_singleline(&mut state.packet_templates[idx].description);
                ui.end_row();

                ui.label(format!("{}:", Tr::header_hex(lang)));
                ui.add(
                    egui::TextEdit::singleline(&mut state.packet_templates[idx].header_hex)
                        .desired_width(120.0),
                );
                ui.label(format!("{}:", Tr::tail_hex(lang)));
                ui.add(
                    egui::TextEdit::singleline(&mut state.packet_templates[idx].tail_hex)
                        .desired_width(120.0),
                );
                ui.end_row();

                ui.label(format!("{}:", Tr::checksum(lang)));
                let current_cs = state.packet_templates[idx].checksum_type;
                egui::ComboBox::from_id_salt("checksum_combo")
                    .selected_text(format!("{}", current_cs))
                    .width(150.0)
                    .show_ui(ui, |ui| {
                        for &cs in ChecksumType::all() {
                            ui.selectable_value(
                                &mut state.packet_templates[idx].checksum_type,
                                cs,
                                format!("{}", cs),
                            );
                        }
                    });
                ui.checkbox(
                    &mut state.packet_templates[idx].include_length,
                    Tr::include_length(lang),
                );
                ui.end_row();
            });
    });

    ui.add_space(10.0);

    // ─── 字段列表 ────────────────────────────────────────
    settings_card(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(Tr::fields(lang)).size(15.0).strong());
            ui.add_space(12.0);
            if ui.button(Tr::add_field(lang)).clicked() {
                state.packet_templates[idx]
                    .fields
                    .push(PacketField::default());
            }
        });
        ui.add_space(8.0);

        let mut remove_idx: Option<usize> = None;

        ScrollArea::vertical()
            .max_height(220.0)
            .id_salt("builder_fields_scroll")
            .show(ui, |ui| {
                let fields_len = state.packet_templates[idx].fields.len();
                for fi in 0..fields_len {
                    egui::Frame::new()
                        .fill(Color32::from_rgba_premultiplied(40, 40, 55, 200))
                        .corner_radius(6.0)
                        .inner_margin(10.0)
                        .outer_margin(egui::Margin::symmetric(0, 3))
                        .show(ui, |ui| {
                            ui.horizontal_wrapped(|ui| {
                                ui.spacing_mut().item_spacing.x = 8.0;
                                ui.checkbox(
                                    &mut state.packet_templates[idx].fields[fi].enabled,
                                    "",
                                );
                                ui.add(
                                    egui::TextEdit::singleline(
                                        &mut state.packet_templates[idx].fields[fi].name,
                                    )
                                    .desired_width(90.0),
                                );

                                let ft = state.packet_templates[idx].fields[fi].field_type;
                                egui::ComboBox::from_id_salt(format!("ft_{}", fi))
                                    .selected_text(format!("{}", ft))
                                    .width(90.0)
                                    .show_ui(ui, |ui| {
                                        for &t in FieldType::all() {
                                            ui.selectable_value(
                                                &mut state.packet_templates[idx].fields[fi]
                                                    .field_type,
                                                t,
                                                format!("{}", t),
                                            );
                                        }
                                    });

                                let en = state.packet_templates[idx].fields[fi].endianness;
                                egui::ComboBox::from_id_salt(format!("en_{}", fi))
                                    .selected_text(format!("{}", en))
                                    .width(55.0)
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(
                                            &mut state.packet_templates[idx].fields[fi].endianness,
                                            Endianness::Little,
                                            "LE",
                                        );
                                        ui.selectable_value(
                                            &mut state.packet_templates[idx].fields[fi].endianness,
                                            Endianness::Big,
                                            "BE",
                                        );
                                    });

                                ui.add(
                                    egui::TextEdit::singleline(
                                        &mut state.packet_templates[idx].fields[fi].value_str,
                                    )
                                    .desired_width(110.0),
                                );

                                if ui.button("Remove").clicked() {
                                    remove_idx = Some(fi);
                                }
                            });
                        });
                }
            });

        if let Some(ri) = remove_idx {
            if state.packet_templates[idx].fields.len() > 1 {
                state.packet_templates[idx].fields.remove(ri);
            }
        }
    });

    ui.add_space(10.0);

    // ─── 构建预览 ────────────────────────────────────────
    let built = state.packet_templates[idx].build();
    let hex_str = bytes_to_hex(&built);
    settings_card(ui, |ui| {
        ui.label(RichText::new(Tr::packet_preview(lang)).size(15.0).strong());
        ui.add_space(8.0);
        ui.label(
            RichText::new(&hex_str)
                .size(14.0)
                .monospace()
                .color(Color32::from_rgb(0, 255, 160)),
        );
        ui.add_space(4.0);
        ui.label(
            RichText::new(format!("{} bytes", built.len()))
                .size(11.5)
                .color(Color32::GRAY),
        );
    });

    ui.add_space(10.0);

    // ─── 操作按钮 ────────────────────────────────────────
    settings_card(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 12.0;

            if ui
                .button(RichText::new(Tr::send_packet(lang)).size(14.0))
                .clicked()
            {
                let data = state.packet_templates[state.ui.packet_template_idx].build();
                match state.send_data(&data) {
                    Ok(()) => state.status_message = Tr::sent_bytes(data.len(), lang),
                    Err(e) => state.status_message = Tr::send_error(&e, lang),
                }
            }

            if ui
                .button(RichText::new(Tr::copy_hex(lang)).size(13.0))
                .clicked()
            {
                ui.ctx().copy_text(hex_str.clone());
                state.status_message = Tr::copied(lang).into();
            }
        });
    });
}

// ═══════════════════════════════════════════════════════════════
// Parser 标签页
// ═══════════════════════════════════════════════════════════════

fn show_parser(ui: &mut Ui, state: &mut AppState) {
    let lang = state.lang();

    // ─── 解析模板选择 ─────────────────────────────────────
    settings_card(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 10.0;
            ui.label(RichText::new(format!("{}:", Tr::parser_template(lang))).strong());
            let names: Vec<String> = state
                .packet_templates
                .iter()
                .map(|t| t.name.clone())
                .collect();
            egui::ComboBox::from_id_salt("parser_template_sel")
                .selected_text(if state.ui.parser_template_idx < names.len() {
                    &names[state.ui.parser_template_idx]
                } else {
                    Tr::select(lang)
                })
                .width(220.0)
                .show_ui(ui, |ui| {
                    for (i, name) in names.iter().enumerate() {
                        ui.selectable_value(&mut state.ui.parser_template_idx, i, name);
                    }
                });

            ui.checkbox(&mut state.ui.parser_auto_parse, Tr::auto_parse(lang));
        });

        ui.add_space(8.0);

        // ─── HEX 数据输入区 ──────────────────────────────────
        ui.label(RichText::new(Tr::parser_input(lang)).size(15.0).strong());
        ui.add_space(4.0);

        egui::Frame::new()
            .fill(Color32::from_rgb(22, 28, 38))
            .corner_radius(6.0)
            .inner_margin(10.0)
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut state.ui.parser_hex_input)
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY)
                        .desired_rows(3)
                        .hint_text(Tr::hex_hint(lang)),
                );
            });

        ui.add_space(8.0);

        // ─── 解析按钮 ────────────────────────────────────────
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 12.0;

            let manual_parse_clicked = ui
                .button(RichText::new(Tr::parse_now(lang)).size(14.0))
                .clicked();
            let current_input = state.ui.parser_hex_input.trim();
            let auto_parse_triggered = state.ui.parser_auto_parse
                && !current_input.is_empty()
                && (state.ui.parser_last_auto_input != current_input
                    || state.ui.parser_last_auto_template_idx != state.ui.parser_template_idx);

            if manual_parse_clicked || auto_parse_triggered {
                do_parse(state);
                state.ui.parser_last_auto_input = state.ui.parser_hex_input.trim().to_string();
                state.ui.parser_last_auto_template_idx = state.ui.parser_template_idx;
            }

            if ui
                .button(RichText::new(Tr::clear(lang)).size(13.0))
                .clicked()
            {
                state.parsed_packets.clear();
                state.ui.parser_last_auto_input.clear();
            }

            ui.label(
                RichText::new(format!(
                    "{}: {}",
                    Tr::parsed_count(lang),
                    state.parsed_packets.len()
                ))
                .size(12.0)
                .color(Color32::GRAY),
            );
        });
    });

    ui.add_space(10.0);

    // ─── 解析结果展示 ─────────────────────────────────────
    if state.parsed_packets.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            ui.label(
                RichText::new(Tr::parser_empty(lang))
                    .size(14.0)
                    .color(Color32::GRAY),
            );
            ui.add_space(40.0);
        });
        return;
    }

    let mut quick_add_field: Option<(String, String)> = None;

    ScrollArea::vertical()
        .max_height(ui.available_height() - 8.0)
        .id_salt("parsed_packets_scroll")
        .show(ui, |ui| {
            // Show most recent first
            for (pi, pkt) in state.parsed_packets.iter().rev().enumerate() {
                let header_text = format!(
                    "#{} {} [{}]  Checksum: {}",
                    state.parsed_packets.len() - pi,
                    pkt.timestamp,
                    pkt.template_name,
                    if pkt.checksum_ok { "OK" } else { "FAIL" },
                );

                egui::CollapsingHeader::new(RichText::new(&header_text).size(13.0).color(
                    if pkt.checksum_ok {
                        Color32::from_rgb(120, 220, 120)
                    } else {
                        Color32::from_rgb(220, 100, 100)
                    },
                ))
                .id_salt(format!("parsed_pkt_{}", pi))
                .default_open(pi == 0) // Most recent expanded
                .show(ui, |ui| {
                    // Raw hex
                    let raw_hex: String = pkt
                        .raw
                        .iter()
                        .map(|b| format!("{:02X}", b))
                        .collect::<Vec<_>>()
                        .join(" ");
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("RAW:").size(11.0).color(Color32::GRAY));
                        ui.label(
                            RichText::new(&raw_hex)
                                .size(11.0)
                                .monospace()
                                .color(Color32::from_rgb(0, 200, 160)),
                        );
                    });
                    ui.add_space(4.0);

                    // Fields table
                    egui::ScrollArea::horizontal().show(ui, |ui| {
                        egui::Grid::new(format!("parsed_fields_grid_{}", pi))
                            .num_columns(5)
                            .striped(true)
                            .spacing([12.0, 4.0])
                            .show(ui, |ui| {
                                // Header row
                                ui.label(
                                    RichText::new(Tr::name(lang))
                                        .size(11.5)
                                        .strong()
                                        .color(Color32::from_rgb(180, 180, 255)),
                                );
                                ui.label(
                                    RichText::new(Tr::field_type_label(lang))
                                        .size(11.5)
                                        .strong()
                                        .color(Color32::from_rgb(180, 180, 255)),
                                );
                                ui.label(
                                    RichText::new(Tr::field_value_label(lang))
                                        .size(11.5)
                                        .strong()
                                        .color(Color32::from_rgb(180, 180, 255)),
                                );
                                ui.label(
                                    RichText::new(Tr::field_numeric(lang))
                                        .size(11.5)
                                        .strong()
                                        .color(Color32::from_rgb(180, 180, 255)),
                                );
                                ui.label(
                                    RichText::new("HEX")
                                        .size(11.5)
                                        .strong()
                                        .color(Color32::from_rgb(180, 180, 255)),
                                );
                                ui.end_row();

                                for field in &pkt.fields {
                                    ui.label(RichText::new(&field.name).size(12.0));
                                    ui.label(
                                        RichText::new(format!("{}", field.field_type))
                                            .size(12.0)
                                            .color(Color32::from_rgb(150, 180, 220)),
                                    );
                                    ui.label(
                                        RichText::new(&field.value_str).size(12.0).monospace(),
                                    );
                                    ui.label(
                                        RichText::new(
                                            field
                                                .value_f64
                                                .map(|v| format!("{:.4}", v))
                                                .unwrap_or_else(|| "-".into()),
                                        )
                                        .size(12.0)
                                        .monospace()
                                        .color(Color32::from_rgb(255, 200, 100)),
                                    );
                                    let field_hex: String = field
                                        .raw_bytes
                                        .iter()
                                        .map(|b| format!("{:02X}", b))
                                        .collect::<Vec<_>>()
                                        .join(" ");
                                    ui.label(
                                        RichText::new(&field_hex)
                                            .size(11.0)
                                            .monospace()
                                            .color(Color32::GRAY),
                                    );
                                    ui.end_row();

                                    if field.value_f64.is_some() {
                                        let btn_text = Tr::viz_add_channel(lang).to_string();
                                        if ui.small_button(btn_text).clicked() {
                                            quick_add_field = Some((
                                                pkt.template_name.clone(),
                                                field.name.clone(),
                                            ));
                                        }
                                        ui.end_row();
                                    }
                                }
                            });
                    });
                });

                ui.add_space(4.0);
            }
        });

    if let Some((template_name, field_name)) = quick_add_field {
        state.add_channel_from_parsed_field(
            &template_name,
            &field_name,
            crate::models::data_channel::VizType::Line,
        );
        state.status_message = format!(
            "Linked packet field to viz: {}/{}",
            template_name, field_name
        );
    }
}

/// Parse the hex input and add result to parsed_packets
fn do_parse(state: &mut AppState) {
    let input = state.ui.parser_hex_input.trim().to_string();
    if input.is_empty() {
        return;
    }

    let data = parse_hex_input(&input);
    if data.is_empty() {
        return;
    }

    let tidx = state.ui.parser_template_idx;
    let result = if tidx < state.packet_templates.len() {
        // Try specific template first, then fallback to auto
        let tmpl = state.packet_templates[tidx].clone();
        state
            .packet_parser
            .parse_with_template(&data, &tmpl)
            .or_else(|| state.packet_parser.try_parse(&data))
    } else {
        state.packet_parser.try_parse(&data)
    };

    if let Some(parsed) = result {
        let lang = state.lang();
        state.status_message = Tr::parse_success(&parsed.template_name, parsed.fields.len(), lang);
        state.feed_parsed_to_channels(&parsed);
        state.parsed_packets.push(parsed);
        // Keep at most 200 parsed packets
        if state.parsed_packets.len() > 200 {
            state.parsed_packets.remove(0);
        }
    } else {
        let lang = state.lang();
        state.status_message = Tr::parse_failed(lang).into();
    }
}

/// Parse a hex string (space-separated or continuous) into bytes
fn parse_hex_input(input: &str) -> Vec<u8> {
    let cleaned: String = input
        .chars()
        .filter(|c| c.is_ascii_hexdigit() || c.is_whitespace())
        .collect();

    // Try space-separated first
    let parts: Vec<&str> = cleaned.split_whitespace().collect();
    let mut bytes = Vec::new();

    for part in &parts {
        if part.len() == 2 {
            if let Ok(b) = u8::from_str_radix(part, 16) {
                bytes.push(b);
                continue;
            }
        }
        // Continuous hex string
        bytes.clear();
        let hex_only: String = cleaned.chars().filter(|c| c.is_ascii_hexdigit()).collect();
        for chunk in hex_only.as_bytes().chunks(2) {
            if chunk.len() == 2 {
                let s = std::str::from_utf8(chunk).unwrap_or("00");
                if let Ok(b) = u8::from_str_radix(s, 16) {
                    bytes.push(b);
                }
            }
        }
        return bytes;
    }

    bytes
}
