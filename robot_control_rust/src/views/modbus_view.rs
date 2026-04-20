use crate::app::AppState;
use crate::i18n::Tr;
use crate::models::modbus::ModbusFunction;
use crate::models::packet::bytes_to_hex;
use crate::views::ui_kit::{page_header, settings_card};
use egui::{self, Color32, RichText, ScrollArea, Ui};

pub fn show(ui: &mut Ui, state: &mut AppState) {
    let lang = state.lang();
    page_header(ui, Tr::tab_modbus(lang), "modbus");

    // ─── 帧构造参数 ──────────────────────────────────────
    settings_card(ui, |ui| {
        ui.label(RichText::new(Tr::request_builder(lang)).size(15.0).strong());
        ui.add_space(8.0);

        egui::Grid::new("modbus_params_grid")
            .num_columns(2)
            .spacing([16.0, 8.0])
            .show(ui, |ui| {
                ui.label(format!("{}:", Tr::slave_id(lang)));
                ui.add(
                    egui::TextEdit::singleline(&mut state.ui.modbus_slave_id_text)
                        .desired_width(90.0),
                );
                ui.end_row();

                ui.label(format!("{}:", Tr::function(lang)));
                let fns = ModbusFunction::all();
                let current_fn = fns[state.ui.modbus_fn_idx.min(fns.len() - 1)];
                egui::ComboBox::from_id_salt("modbus_fn_combo")
                    .selected_text(format!("{}", current_fn))
                    .width(ui.available_width().clamp(180.0, 320.0))
                    .show_ui(ui, |ui| {
                        for (i, f) in fns.iter().enumerate() {
                            ui.selectable_value(&mut state.ui.modbus_fn_idx, i, format!("{}", f));
                        }
                    });
                ui.end_row();

                ui.label(format!("{}:", Tr::start_address(lang)));
                ui.add(
                    egui::TextEdit::singleline(&mut state.ui.modbus_start_addr_text)
                        .desired_width(110.0),
                );
                ui.end_row();

                ui.label(format!("{}:", Tr::quantity(lang)));
                ui.add(
                    egui::TextEdit::singleline(&mut state.ui.modbus_quantity_text)
                        .desired_width(110.0),
                );
                ui.end_row();

                let fn_idx = state.ui.modbus_fn_idx.min(fns.len() - 1);
                let selected_fn = fns[fn_idx];
                if !selected_fn.is_read() {
                    ui.label(format!("{}:", Tr::write_values(lang)));
                    ui.add(
                        egui::TextEdit::singleline(&mut state.ui.modbus_write_values_text)
                            .desired_width(320.0),
                    )
                    .on_hover_text(Tr::comma_values_hint(lang));
                    ui.end_row();
                }
            });
    });

    // 更新 ModbusFrame
    let fns = ModbusFunction::all();
    let fn_idx = state.ui.modbus_fn_idx.min(fns.len() - 1);
    state.modbus_frame.slave_id = state.ui.modbus_slave_id_text.parse().unwrap_or(1);
    state.modbus_frame.function = fns[fn_idx];
    state.modbus_frame.start_address = state.ui.modbus_start_addr_text.parse().unwrap_or(0);
    state.modbus_frame.quantity = state.ui.modbus_quantity_text.parse().unwrap_or(10);

    if !fns[fn_idx].is_read() {
        state.modbus_frame.write_values = state
            .ui
            .modbus_write_values_text
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect();
    }

    ui.add_space(10.0);

    // ─── 帧预览 ─────────────────────────────────────────
    let rtu_frame = state.modbus_frame.build_rtu_request();
    let tcp_frame = state.modbus_frame.build_tcp_request(1);

    settings_card(ui, |ui| {
        ui.label(RichText::new(Tr::frame_preview(lang)).size(15.0).strong());
        ui.add_space(8.0);
        ui.label(
            RichText::new("RTU:")
                .size(12.0)
                .strong()
                .color(Color32::from_rgb(255, 165, 0)),
        );
        ui.label(
            RichText::new(bytes_to_hex(&rtu_frame))
                .monospace()
                .color(Color32::from_rgb(0, 255, 160)),
        );
        ui.label(
            RichText::new(format!("{} bytes", rtu_frame.len()))
                .size(11.5)
                .color(Color32::GRAY),
        );
        ui.add_space(6.0);
        ui.label(
            RichText::new("TCP (MBAP):")
                .size(12.0)
                .strong()
                .color(Color32::from_rgb(100, 200, 255)),
        );
        ui.label(
            RichText::new(bytes_to_hex(&tcp_frame))
                .monospace()
                .color(Color32::from_rgb(0, 255, 160)),
        );
        ui.label(
            RichText::new(format!("{} bytes", tcp_frame.len()))
                .size(11.5)
                .color(Color32::GRAY),
        );
    });

    ui.add_space(10.0);

    // ─── 发送按钮 ────────────────────────────────────────
    settings_card(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 12.0;

            if ui
                .button(RichText::new(Tr::send_rtu(lang)).size(14.0))
                .clicked()
            {
                let data = state.modbus_frame.build_rtu_request();
                match state.send_data(&data) {
                    Ok(()) => {
                        state
                            .modbus_response_log
                            .push(format!("[TX RTU] {}", bytes_to_hex(&data)));
                        state.status_message = Tr::sent_bytes(data.len(), lang);
                    }
                    Err(e) => state.status_message = Tr::send_error(&e, lang),
                }
            }

            if ui
                .button(RichText::new(Tr::send_tcp(lang)).size(14.0))
                .clicked()
            {
                let data = state.modbus_frame.build_tcp_request(1);
                match state.send_data(&data) {
                    Ok(()) => {
                        state
                            .modbus_response_log
                            .push(format!("[TX TCP] {}", bytes_to_hex(&data)));
                        state.status_message = Tr::sent_bytes(data.len(), lang);
                    }
                    Err(e) => state.status_message = Tr::send_error(&e, lang),
                }
            }

            if ui.button("Copy RTU").clicked() {
                ui.ctx().copy_text(bytes_to_hex(&rtu_frame));
            }

            if ui.button("Copy TCP").clicked() {
                ui.ctx().copy_text(bytes_to_hex(&tcp_frame));
            }
        });
    });

    ui.add_space(10.0);

    // ─── 模拟寄存器表 ────────────────────────────────────
    settings_card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new(Tr::register_table(lang)).size(15.0).strong());
            ui.add_space(12.0);
            if ui.button(Tr::randomize(lang)).clicked() {
                for reg in state.modbus_registers.iter_mut() {
                    *reg = (*reg).wrapping_add(1);
                }
            }
        });
        ui.add_space(8.0);

        ScrollArea::both().max_height(180.0).show(ui, |ui| {
            egui::Grid::new("register_table")
                .num_columns(9)
                .spacing([10.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label(RichText::new("Addr").strong().size(11.5));
                    for j in 0..8 {
                        ui.label(RichText::new(format!("+{}", j)).strong().size(11.5));
                    }
                    ui.end_row();

                    let start = state.modbus_frame.start_address as usize;
                    let qty = state.modbus_frame.quantity as usize;
                    let end = (start + qty).min(state.modbus_registers.len());
                    let range_start = (start / 8) * 8;
                    let range_end = end.div_ceil(8) * 8;

                    for row_start in
                        (range_start..range_end.min(state.modbus_registers.len())).step_by(8)
                    {
                        ui.label(
                            RichText::new(format!("{:05}", row_start))
                                .monospace()
                                .size(11.5),
                        );
                        for j in 0..8 {
                            let addr = row_start + j;
                            if addr < state.modbus_registers.len() {
                                let in_range = addr >= start && addr < end;
                                let text = format!("{:5}", state.modbus_registers[addr]);
                                let rt = RichText::new(text).monospace().size(11.5);
                                if in_range {
                                    ui.label(rt.color(Color32::from_rgb(100, 255, 100)));
                                } else {
                                    ui.label(rt.color(Color32::GRAY));
                                }
                            } else {
                                ui.label("  -  ");
                            }
                        }
                        ui.end_row();
                    }
                });
        });
    });

    // ─── 日志 ────────────────────────────────────────────
    if !state.modbus_response_log.is_empty() {
        ui.add_space(10.0);
        settings_card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(Tr::modbus_log(lang)).size(15.0).strong());
                ui.add_space(12.0);
                if ui.button(Tr::clear(lang)).clicked() {
                    state.modbus_response_log.clear();
                }
            });
            ui.add_space(6.0);

            ScrollArea::vertical().max_height(110.0).show(ui, |ui| {
                for entry in state.modbus_response_log.iter().rev().take(50) {
                    ui.label(RichText::new(entry).size(11.5).monospace());
                }
            });
        });
    }
}
