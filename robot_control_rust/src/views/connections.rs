use crate::app::AppState;
use crate::i18n::Tr;
use crate::models::*;
use crate::views::ui_kit::{page_header, settings_card};
use egui::{self, Color32, RichText, Ui};

const CONN_LABEL_WIDTH: f32 = 140.0;
const CONN_INPUT_WIDTH: f32 = 240.0;

pub fn show(ui: &mut Ui, state: &mut AppState) {
    let lang = state.lang();
    page_header(ui, Tr::tab_connections(lang), "connections");

    settings_card(ui, |ui| {
        ui.label(RichText::new(Tr::protocol(lang)).size(14.0).strong());
        ui.add_space(6.0);

        let conn_types = ConnectionType::all();
        let combo_w = ui.available_width().clamp(180.0, 300.0);
        egui::ComboBox::from_id_salt("conn_type_combo")
            .selected_text(format!("{}", conn_types[state.ui.conn_type_idx]))
            .width(combo_w)
            .show_ui(ui, |ui| {
                for (i, ct) in conn_types.iter().enumerate() {
                    ui.selectable_value(&mut state.ui.conn_type_idx, i, format!("{}", ct));
                }
            });

        state.active_conn = conn_types[state.ui.conn_type_idx];
    });

    ui.add_space(10.0);

    settings_card(ui, |ui| match state.active_conn {
        ConnectionType::Serial => show_serial_config(ui, state),
        ConnectionType::Usb => show_usb_config(ui, state),
        ConnectionType::Tcp | ConnectionType::ModbusTcp => show_tcp_config(ui, state),
        ConnectionType::Udp => show_udp_config(ui, state),
        ConnectionType::Can | ConnectionType::CanFd => show_can_config(ui, state),
        ConnectionType::ModbusRtu => show_modbus_rtu_config(ui, state),
    });

    ui.add_space(10.0);

    settings_card(ui, |ui| {
        ui.label(RichText::new("MCP Server").size(15.0).strong());
        ui.add_space(8.0);

        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(10.0, 8.0);
            ui.label("Port:");
            ui.add(egui::TextEdit::singleline(&mut state.ui.mcp_port_text).desired_width(100.0));
            ui.label("Token:");
            ui.add(
                egui::TextEdit::singleline(&mut state.ui.mcp_token_text)
                    .password(true)
                    .desired_width(170.0),
            );

            if state.ui.mcp_running {
                if ui
                    .button(RichText::new("Stop MCP").color(Color32::from_rgb(255, 120, 120)))
                    .clicked()
                {
                    state.stop_mcp_server();
                }
                ui.label(RichText::new("Running").color(Color32::from_rgb(120, 220, 120)));
            } else {
                if ui
                    .button(RichText::new("Start MCP").color(Color32::from_rgb(120, 220, 120)))
                    .clicked()
                {
                    if let Err(e) = state.start_mcp_server() {
                        state.report_error(format!("MCP start failed: {}", e));
                    }
                }
                ui.label(RichText::new("Stopped").color(Color32::GRAY));
            }
        });
    });

    ui.add_space(10.0);

    let connected = state.active_status().is_connected();
    settings_card(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 16.0;
            if connected {
                if ui
                    .button(
                        RichText::new(Tr::disconnect(lang))
                            .size(15.0)
                            .color(Color32::from_rgb(255, 100, 100)),
                    )
                    .clicked()
                {
                    state.disconnect_active();
                    state.status_message =
                        format!("{} {}", state.active_conn, Tr::disconnected(lang));
                }
                let (r, g, b) = state.active_status().color_rgb();
                ui.label(
                    RichText::new(format!("{}", state.active_status()))
                        .color(Color32::from_rgb(r, g, b)),
                );
            } else {
                if ui
                    .button(
                        RichText::new(Tr::connect(lang))
                            .size(15.0)
                            .color(Color32::from_rgb(100, 200, 100)),
                    )
                    .clicked()
                {
                    match state.connect_active() {
                        Ok(()) => {
                            state.status_message =
                                format!("{} {}!", state.active_conn, Tr::connected(lang))
                        }
                        Err(e) => state.report_error(format!("{}: {}", Tr::error_label(lang), e)),
                    }
                }
                ui.label(RichText::new(Tr::disconnected(lang)).color(Color32::GRAY));
            }

            ui.separator();
            ui.checkbox(&mut state.ui.auto_reconnect_enabled, "Auto reconnect");
            ui.label("Interval(ms):");
            ui.add(
                egui::DragValue::new(&mut state.ui.auto_reconnect_interval_ms)
                    .range(500..=30000)
                    .speed(50.0),
            );
            if state.reconnect_paused()
                && state.ui.auto_reconnect_enabled
                && ui.button("Resume reconnect").clicked()
            {
                state.resume_auto_reconnect();
            }
        });
    });
}

fn show_serial_config(ui: &mut Ui, state: &mut AppState) {
    let lang = state.lang();
    ui.label(RichText::new(Tr::serial_config(lang)).size(15.0).strong());
    ui.add_space(10.0);

    // 端口选择
    ui.horizontal_wrapped(|ui| {
        ui.label(format!("{}:", Tr::port(lang)));
        egui::ComboBox::from_id_salt("serial_port_combo")
            .selected_text(if state.serial.config.port_name.is_empty() {
                Tr::select_port(lang)
            } else {
                &state.serial.config.port_name
            })
            .width(220.0)
            .show_ui(ui, |ui| {
                for port in &state.available_ports {
                    ui.selectable_value(&mut state.serial.config.port_name, port.clone(), port);
                }
            });
        if ui.button(Tr::refresh(lang)).clicked() {
            state.refresh_ports();
        }
        if state.serial.config.port_name.trim().is_empty()
            && !state.available_ports.is_empty()
            && ui.button("Use first").clicked()
        {
            state.serial.config.port_name = state.available_ports[0].clone();
        }
    });

    if !state.serial.config.port_name.trim().is_empty() {
        let info = crate::services::SerialService::get_port_info(&state.serial.config.port_name);
        ui.add_space(4.0);
        ui.label(
            RichText::new(format!("Port info: {}", info))
                .small()
                .color(Color32::GRAY),
        );
    }

    ui.add_space(8.0);

    if (state.active_conn == ConnectionType::Serial
        || state.active_conn == ConnectionType::Usb
        || state.active_conn == ConnectionType::ModbusRtu)
        && state.available_ports.is_empty()
    {
        ui.label(
            RichText::new("Port scan may still be running...")
                .small()
                .color(Color32::GRAY),
        );
    }

    // 波特率
    ui.horizontal_wrapped(|ui| {
        ui.label(format!("{}:", Tr::baud_rate(lang)));
        let bauds = SerialConfig::baud_rates();
        egui::ComboBox::from_id_salt("baud_combo")
            .selected_text(format!("{}", state.serial.config.baud_rate))
            .width(130.0)
            .show_ui(ui, |ui| {
                for (i, &baud) in bauds.iter().enumerate() {
                    if ui
                        .selectable_value(&mut state.ui.serial_baud_idx, i, format!("{}", baud))
                        .clicked()
                    {
                        state.serial.config.baud_rate = baud;
                    }
                }
            });
    });

    ui.add_space(10.0);

    egui::Grid::new("serial_params_grid")
        .num_columns(2)
        .spacing([20.0, 12.0])
        .show(ui, |ui| {
            // 数据位
            ui.label(format!("{}:", Tr::data_bits(lang)));
            egui::ComboBox::from_id_salt("databits_combo")
                .selected_text(format!("{}", state.serial.config.data_bits))
                .width(70.0)
                .show_ui(ui, |ui| {
                    for &d in SerialConfig::data_bits_options() {
                        ui.selectable_value(
                            &mut state.serial.config.data_bits,
                            d,
                            format!("{}", d),
                        );
                    }
                });

            ui.end_row();

            // 停止位
            ui.label(format!("{}:", Tr::stop_bits(lang)));
            egui::ComboBox::from_id_salt("stopbits_combo")
                .selected_text(format!("{}", state.serial.config.stop_bits))
                .width(70.0)
                .show_ui(ui, |ui| {
                    for &s in SerialConfig::stop_bits_options() {
                        ui.selectable_value(
                            &mut state.serial.config.stop_bits,
                            s,
                            format!("{}", s),
                        );
                    }
                });
            ui.end_row();

            // 校验
            ui.label(format!("{}:", Tr::parity(lang)));
            egui::ComboBox::from_id_salt("parity_combo")
                .selected_text(&state.serial.config.parity)
                .width(110.0)
                .show_ui(ui, |ui| {
                    for &p in SerialConfig::parity_options() {
                        ui.selectable_value(&mut state.serial.config.parity, p.to_string(), p);
                    }
                });

            ui.end_row();

            // 流控
            ui.label(format!("{}:", Tr::flow_control(lang)));
            egui::ComboBox::from_id_salt("flow_combo")
                .selected_text(&state.serial.config.flow_control)
                .width(170.0)
                .show_ui(ui, |ui| {
                    for &fc in SerialConfig::flow_control_options() {
                        ui.selectable_value(
                            &mut state.serial.config.flow_control,
                            fc.to_string(),
                            fc,
                        );
                    }
                });
            ui.end_row();
        });

    // 可用端口列表
    ui.add_space(12.0);
    ui.collapsing(Tr::available_ports(lang), |ui| {
        if state.available_ports.is_empty() {
            ui.label(Tr::no_ports_found(lang));
        } else {
            for port in &state.available_ports {
                ui.label(format!("- {}", port));
            }
        }
    });
}

fn show_tcp_config(ui: &mut Ui, state: &mut AppState) {
    let lang = state.lang();
    ui.label(RichText::new(Tr::tcp_config(lang)).size(15.0).strong());
    ui.add_space(10.0);

    egui::Grid::new("tcp_config_grid")
        .num_columns(2)
        .spacing([20.0, 12.0])
        .show(ui, |ui| {
            ui.add_sized(
                [CONN_LABEL_WIDTH, 20.0],
                egui::Label::new(format!("{}:", Tr::mode(lang))),
            );
            ui.horizontal(|ui| {
                ui.selectable_value(&mut state.ui.tcp_is_server, false, Tr::client(lang));
                ui.selectable_value(&mut state.ui.tcp_is_server, true, Tr::server(lang));
            });
            ui.end_row();

            ui.add_sized(
                [CONN_LABEL_WIDTH, 20.0],
                egui::Label::new(format!("{}:", Tr::host(lang))),
            );
            ui.add(
                egui::TextEdit::singleline(&mut state.ui.tcp_host).desired_width(CONN_INPUT_WIDTH),
            );
            ui.end_row();

            ui.add_sized(
                [CONN_LABEL_WIDTH, 20.0],
                egui::Label::new(format!("{}:", Tr::port(lang))),
            );
            ui.add(egui::TextEdit::singleline(&mut state.ui.tcp_port_text).desired_width(120.0));
            ui.end_row();
        });

    if state.tcp.is_connected() && !state.tcp.connected_clients.is_empty() {
        ui.add_space(8.0);
        ui.collapsing(Tr::connected_clients(lang), |ui| {
            for client in &state.tcp.connected_clients {
                ui.label(format!("- {}", client));
            }
        });
    }
}

fn show_udp_config(ui: &mut Ui, state: &mut AppState) {
    let lang = state.lang();
    ui.label(RichText::new(Tr::udp_config(lang)).size(15.0).strong());
    ui.add_space(10.0);

    egui::Grid::new("udp_config_grid")
        .num_columns(2)
        .spacing([20.0, 12.0])
        .show(ui, |ui| {
            ui.add_sized(
                [CONN_LABEL_WIDTH, 20.0],
                egui::Label::new(format!("{}:", Tr::local_port(lang))),
            );
            ui.add(
                egui::TextEdit::singleline(&mut state.ui.udp_local_port_text).desired_width(120.0),
            );
            ui.end_row();

            ui.add_sized(
                [CONN_LABEL_WIDTH, 20.0],
                egui::Label::new(format!("{}:", Tr::remote_host(lang))),
            );
            ui.add(
                egui::TextEdit::singleline(&mut state.ui.udp_remote_host)
                    .desired_width(CONN_INPUT_WIDTH),
            );
            ui.end_row();

            ui.add_sized(
                [CONN_LABEL_WIDTH, 20.0],
                egui::Label::new(format!("{}:", Tr::remote_port(lang))),
            );
            ui.add(
                egui::TextEdit::singleline(&mut state.ui.udp_remote_port_text).desired_width(120.0),
            );
            ui.end_row();
        });
}

fn show_can_config(ui: &mut Ui, state: &mut AppState) {
    let lang = state.lang();
    ui.label(RichText::new(Tr::can_config(lang)).size(15.0).strong());
    ui.add_space(4.0);
    ui.label(
        RichText::new(Tr::sw_simulation_hint(lang))
            .size(11.0)
            .color(Color32::GRAY),
    );
    ui.add_space(10.0);

    // ─── 仲裁段波特率 ────────────────────────────────────
    ui.horizontal(|ui| {
        ui.add_sized(
            [CONN_LABEL_WIDTH, 20.0],
            egui::Label::new(RichText::new(format!("{}:", Tr::bitrate(lang))).strong()),
        );
        let bitrates = CanConfig::standard_bitrates();
        let selected_text = bitrates
            .get(state.ui.can_bitrate_idx)
            .map(|(_, label)| label.to_string())
            .unwrap_or_else(|| format!("{} kbps", state.can.bitrate / 1000));
        egui::ComboBox::from_id_salt("can_arb_bitrate")
            .selected_text(&selected_text)
            .width(160.0)
            .show_ui(ui, |ui| {
                for (i, &(rate, label)) in bitrates.iter().enumerate() {
                    if ui
                        .selectable_value(&mut state.ui.can_bitrate_idx, i, label)
                        .clicked()
                    {
                        state.can.bitrate = rate;
                    }
                }
            });
    });

    ui.add_space(4.0);

    // ─── 仲裁段采样点 ────────────────────────────────────
    ui.horizontal_wrapped(|ui| {
        ui.add_sized(
            [CONN_LABEL_WIDTH, 20.0],
            egui::Label::new(format!("{}:", Tr::sample_point(lang))),
        );
        let sp_opts = CanConfig::sample_point_options();
        let sp_text = sp_opts
            .get(state.ui.can_sample_point_idx)
            .map(|(_, label)| label.to_string())
            .unwrap_or_else(|| format!("{:.1}%", state.can.config_sample_point * 100.0));
        egui::ComboBox::from_id_salt("can_arb_sp")
            .selected_text(&sp_text)
            .width(100.0)
            .show_ui(ui, |ui| {
                for (i, &(sp, label)) in sp_opts.iter().enumerate() {
                    if ui
                        .selectable_value(&mut state.ui.can_sample_point_idx, i, label)
                        .clicked()
                    {
                        state.can.config_sample_point = sp;
                    }
                }
            });

        ui.add_space(12.0);
        ui.label("SJW:");
        let sjw_opts = CanConfig::sjw_options();
        let sjw_text = format!(
            "{}",
            sjw_opts.get(state.ui.can_sjw_idx).copied().unwrap_or(1)
        );
        egui::ComboBox::from_id_salt("can_arb_sjw")
            .selected_text(&sjw_text)
            .width(60.0)
            .show_ui(ui, |ui| {
                for (i, &sjw) in sjw_opts.iter().enumerate() {
                    ui.selectable_value(&mut state.ui.can_sjw_idx, i, format!("{}", sjw));
                }
            });
    });

    ui.add_space(8.0);

    // ─── CAN FD 启用 ─────────────────────────────────────
    ui.checkbox(&mut state.can.fd_enabled, Tr::enable_can_fd(lang));

    if state.can.fd_enabled {
        ui.add_space(6.0);

        // ─── 数据段波特率 ────────────────────────────────
        ui.horizontal(|ui| {
            ui.add_sized(
                [CONN_LABEL_WIDTH, 20.0],
                egui::Label::new(RichText::new(format!("{}:", Tr::data_bitrate(lang))).strong()),
            );
            let data_rates = CanConfig::fd_data_bitrates();
            let dr_text = data_rates
                .get(state.ui.can_data_bitrate_idx)
                .map(|(_, label)| label.to_string())
                .unwrap_or_else(|| format!("{} Mbps", state.can.data_bitrate / 1_000_000));
            egui::ComboBox::from_id_salt("can_data_bitrate_v2")
                .selected_text(&dr_text)
                .width(160.0)
                .show_ui(ui, |ui| {
                    for (i, &(rate, label)) in data_rates.iter().enumerate() {
                        if ui
                            .selectable_value(&mut state.ui.can_data_bitrate_idx, i, label)
                            .clicked()
                        {
                            state.can.data_bitrate = rate;
                        }
                    }
                });
        });

        ui.add_space(4.0);

        // ─── 数据段采样点 + SJW ──────────────────────────
        ui.horizontal_wrapped(|ui| {
            ui.add_sized(
                [CONN_LABEL_WIDTH, 20.0],
                egui::Label::new(format!("{}:", Tr::data_sample_point(lang))),
            );
            let sp_opts = CanConfig::sample_point_options();
            let sp_text = sp_opts
                .get(state.ui.can_data_sample_point_idx)
                .map(|(_, label)| label.to_string())
                .unwrap_or("75.0%".into());
            egui::ComboBox::from_id_salt("can_data_sp")
                .selected_text(&sp_text)
                .width(100.0)
                .show_ui(ui, |ui| {
                    for (i, &(_sp, label)) in sp_opts.iter().enumerate() {
                        ui.selectable_value(&mut state.ui.can_data_sample_point_idx, i, label);
                    }
                });

            ui.add_space(12.0);
            ui.label(format!("{} SJW:", Tr::data_bitrate(lang)));
            let sjw_opts = CanConfig::sjw_options();
            let sjw_text = format!(
                "{}",
                sjw_opts
                    .get(state.ui.can_data_sjw_idx)
                    .copied()
                    .unwrap_or(1)
            );
            egui::ComboBox::from_id_salt("can_data_sjw")
                .selected_text(&sjw_text)
                .width(60.0)
                .show_ui(ui, |ui| {
                    for (i, &sjw) in sjw_opts.iter().enumerate() {
                        ui.selectable_value(&mut state.ui.can_data_sjw_idx, i, format!("{}", sjw));
                    }
                });
        });
    }

    ui.add_space(10.0);

    // ─── 高级选项 ────────────────────────────────────────
    ui.collapsing(Tr::advanced_options(lang), |ui| {
        ui.add_space(4.0);
        ui.checkbox(&mut state.can.config_termination, Tr::can_termination(lang));
        ui.checkbox(&mut state.can.config_listen_only, Tr::can_listen_only(lang));
        ui.checkbox(&mut state.can.config_loopback, Tr::can_loopback(lang));
        ui.checkbox(
            &mut state.can.config_auto_retransmit,
            Tr::can_auto_retransmit(lang),
        );
        ui.checkbox(
            &mut state.can.config_error_reporting,
            Tr::can_error_reporting(lang),
        );
    });

    ui.add_space(10.0);

    // ─── 运行状态 ────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.label(format!(
            "TX: {}  |  RX: {}  |  Bus Load: {:.1}%",
            state.can.frame_count_tx, state.can.frame_count_rx, state.can.bus_load
        ));
    });

    ui.add_space(6.0);

    let can_btn_text = if state.can.is_running {
        Tr::stop(lang).to_string()
    } else {
        Tr::start(lang).to_string()
    };
    if ui.button(can_btn_text).clicked() {
        state.can.is_running = !state.can.is_running;
    }
}

fn show_usb_config(ui: &mut Ui, state: &mut AppState) {
    let lang = state.lang();
    ui.label(RichText::new(Tr::usb_config(lang)).size(15.0).strong());
    ui.add_space(10.0);

    // ─── USB 协议选择 ────────────────────────────────────
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new(format!("{}:", Tr::usb_protocol_label(lang))).strong());
        let protocols = UsbProtocol::all();
        let selected = protocols
            .get(state.ui.usb_protocol_idx)
            .copied()
            .unwrap_or(UsbProtocol::CdcAcm);
        egui::ComboBox::from_id_salt("usb_protocol_combo")
            .selected_text(format!("{}", selected))
            .width(ui.available_width().clamp(180.0, 280.0))
            .show_ui(ui, |ui| {
                for (i, &proto) in protocols.iter().enumerate() {
                    if ui
                        .selectable_value(&mut state.ui.usb_protocol_idx, i, format!("{}", proto))
                        .clicked()
                    {
                        state.usb_config.protocol = proto;
                    }
                }
            });
    });

    // 协议描述
    let selected_proto = UsbProtocol::all()
        .get(state.ui.usb_protocol_idx)
        .copied()
        .unwrap_or(UsbProtocol::CdcAcm);
    ui.add_space(4.0);
    ui.label(
        RichText::new(format!(
            "  {} (Class: 0x{:02X})",
            selected_proto.description(),
            selected_proto.class_code()
        ))
        .size(12.0)
        .color(Color32::GRAY)
        .italics(),
    );

    ui.add_space(10.0);

    // ─── USB 速度 ────────────────────────────────────────
    ui.horizontal_wrapped(|ui| {
        ui.label(format!("{}:", Tr::usb_speed_label(lang)));
        let speeds = UsbSpeed::all();
        let spd = speeds
            .get(state.ui.usb_speed_idx)
            .copied()
            .unwrap_or(UsbSpeed::FullSpeed);
        egui::ComboBox::from_id_salt("usb_speed_combo")
            .selected_text(format!("{}", spd))
            .width(240.0)
            .show_ui(ui, |ui| {
                for (i, &speed) in speeds.iter().enumerate() {
                    if ui
                        .selectable_value(&mut state.ui.usb_speed_idx, i, format!("{}", speed))
                        .clicked()
                    {
                        state.usb_config.speed = speed;
                    }
                }
            });
    });

    ui.add_space(10.0);

    // ─── VID / PID ───────────────────────────────────────
    egui::Grid::new("usb_vid_pid_grid")
        .num_columns(4)
        .spacing([12.0, 6.0])
        .show(ui, |ui| {
            ui.label("VID:");
            ui.add(
                egui::TextEdit::singleline(&mut state.ui.usb_vid_text)
                    .desired_width(80.0)
                    .hint_text("0483"),
            );
            ui.label("PID:");
            ui.add(
                egui::TextEdit::singleline(&mut state.ui.usb_pid_text)
                    .desired_width(80.0)
                    .hint_text("5740"),
            );
            ui.end_row();
        });

    ui.add_space(10.0);

    // ─── 端点与包大小 ────────────────────────────────────
    ui.collapsing(Tr::usb_endpoint_config(lang), |ui| {
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            ui.label(format!(
                "{}: 0x{:02X}",
                Tr::usb_endpoint_in(lang),
                state.usb_config.endpoint_in
            ));
            ui.add_space(16.0);
            ui.label(format!(
                "{}: 0x{:02X}",
                Tr::usb_endpoint_out(lang),
                state.usb_config.endpoint_out
            ));
        });
        ui.add_space(4.0);
        ui.label(format!(
            "{}: {} bytes",
            Tr::usb_max_packet_size(lang),
            state.usb_config.max_packet_size
        ));
        ui.label(format!(
            "{}: {}",
            Tr::usb_interface(lang),
            state.usb_config.interface_num
        ));
    });

    // ─── 典型速度提示 ────────────────────────────────────
    ui.add_space(8.0);
    let speeds = selected_proto.typical_speeds();
    if !speeds.is_empty() {
        ui.collapsing(Tr::usb_typical_speeds(lang), |ui| {
            for &s in speeds {
                ui.label(format!("- {}", s));
            }
        });
    }

    // ─── 底层仍复用串口连接（CDC ACM模式下） ─────────────
    if selected_proto == UsbProtocol::CdcAcm {
        ui.add_space(8.0);
        ui.separator();
        ui.add_space(4.0);
        ui.label(
            RichText::new(Tr::usb_cdc_hint(lang))
                .size(12.0)
                .color(Color32::GRAY),
        );
        ui.add_space(4.0);
        // 显示串口选择（CDC ACM 模式复用串口）
        ui.horizontal_wrapped(|ui| {
            ui.label(format!("{}:", Tr::port(lang)));
            egui::ComboBox::from_id_salt("usb_cdc_port_combo")
                .selected_text(if state.serial.config.port_name.is_empty() {
                    Tr::select_port(lang)
                } else {
                    &state.serial.config.port_name
                })
                .width(220.0)
                .show_ui(ui, |ui| {
                    for port in &state.available_ports {
                        ui.selectable_value(&mut state.serial.config.port_name, port.clone(), port);
                    }
                });
            if ui.button(Tr::refresh(lang)).clicked() {
                state.refresh_ports();
            }
        });
    }
}

fn show_modbus_rtu_config(ui: &mut Ui, state: &mut AppState) {
    let lang = state.lang();
    ui.label(RichText::new("Modbus RTU").size(15.0).strong());
    ui.add_space(10.0);

    ui.label(
        RichText::new(if lang == crate::i18n::Language::Chinese {
            "使用串口标签页中的串口参数配置。"
        } else {
            "Uses serial port settings from the Serial tab."
        })
        .size(13.0)
        .color(Color32::GRAY),
    );

    ui.add_space(8.0);
    ui.horizontal_wrapped(|ui| {
        ui.label(format!(
            "{}: {}  |  {}: {}",
            Tr::port(lang),
            if state.serial.config.port_name.is_empty() {
                "(none)"
            } else {
                &state.serial.config.port_name
            },
            Tr::baud_rate(lang),
            state.serial.config.baud_rate
        ));
    });
}
