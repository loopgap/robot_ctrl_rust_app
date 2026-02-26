use crate::app::AppState;
use crate::models::canopen::{
    analyze_canopen_frame, analyze_ecat_coe_frame, build_heartbeat_producer_sdo, build_nmt,
    build_pdo, canopen_id_role, decode_emcy, decode_heartbeat_state, ecat_state_name,
    fd_len_to_dlc, is_fd_valid_len, object_dict_name, preset_pdo_configs, CanProtocolType,
    CanStdFrame, CanopenSdoRequest, EcatCoeSdoRequest, MultiProtocolFrame, NmtCommand, PdoConfig,
    PdoDirection, SdoAction,
};
use crate::models::packet::{bytes_to_hex, parse_hex_string};
use crate::views::ui_kit::{page_header, settings_card};
use egui::{self, Color32, RichText, Ui};

pub fn show(ui: &mut Ui, state: &mut AppState) {
    page_header(ui, "CAN 多协议工具 / Multi-Protocol CAN Tools", "canopen");

    // ═══ 协议选择器 ═══
    settings_card(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("协议 / Protocol").strong().size(13.0));
            let protos = CanProtocolType::all();
            state.ui.canopen_protocol_idx = state.ui.canopen_protocol_idx.min(protos.len() - 1);
            for (i, p) in protos.iter().enumerate() {
                let selected = state.ui.canopen_protocol_idx == i;
                let color = if selected {
                    Color32::from_rgb(60, 140, 240)
                } else {
                    Color32::from_rgb(50, 60, 75)
                };
                egui::Frame::NONE
                    .fill(if selected {
                        Color32::from_rgb(25, 50, 80)
                    } else {
                        Color32::from_rgb(22, 28, 38)
                    })
                    .stroke(egui::Stroke::new(if selected { 2.0 } else { 1.0 }, color))
                    .corner_radius(6.0)
                    .inner_margin(egui::Margin::symmetric(14, 5))
                    .show(ui, |ui| {
                        if ui
                            .add(
                                egui::Label::new(
                                    RichText::new(p.label()).strong().size(12.0).color(
                                        if selected {
                                            Color32::from_rgb(100, 200, 255)
                                        } else {
                                            Color32::from_rgb(170, 175, 185)
                                        },
                                    ),
                                )
                                .sense(egui::Sense::click()),
                            )
                            .clicked()
                        {
                            state.ui.canopen_protocol_idx = i;
                        }
                    });
            }
        });
    });
    ui.add_space(4.0);

    let active_proto = CanProtocolType::all()[state.ui.canopen_protocol_idx];

    match active_proto {
        CanProtocolType::Standard => show_canopen_standard(ui, state),
        CanProtocolType::Fd => show_can_fd(ui, state),
        CanProtocolType::EtherCatCoE => show_ethercat_coe(ui, state),
    }
}

// ═══════════════════════════════════════════════════════════════
// CAN FD 专用页面
// ═══════════════════════════════════════════════════════════════
fn show_can_fd(ui: &mut Ui, state: &mut AppState) {
    settings_card(ui, |ui| {
        ui.label(
            RichText::new("CAN FD 帧构建 / CAN FD Frame Builder")
                .strong()
                .size(15.0),
        );
        ui.add_space(6.0);

        ui.horizontal_wrapped(|ui| {
            ui.label("CAN ID:");
            ui.add(
                egui::TextEdit::singleline(&mut state.ui.canopen_analyze_cobid_text)
                    .desired_width(100.0),
            );
            ui.checkbox(&mut state.ui.can_extended, "29-bit Extended");
        });

        ui.horizontal_wrapped(|ui| {
            ui.label("FD Data (max 64B):");
            ui.add(
                egui::TextEdit::singleline(&mut state.ui.canopen_fd_data_text)
                    .desired_width(500.0)
                    .hint_text("00 01 02 ... up to 64 bytes"),
            );
        });

        let fd_data = parse_hex_string(&state.ui.canopen_fd_data_text);
        let can_id = parse_u32_text(&state.ui.canopen_analyze_cobid_text).unwrap_or(0x100);
        let frame = CanStdFrame::new_fd(can_id, &fd_data, state.ui.can_extended);

        let valid_len = is_fd_valid_len(fd_data.len());
        let dlc_code = fd_len_to_dlc(fd_data.len());

        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            draw_pdo_badge(ui, "CAN FD", true);
            draw_pdo_badge(ui, &format!("DLC={}", dlc_code), true);
            draw_pdo_badge(ui, &format!("{}B", fd_data.len()), true);
            draw_pdo_badge(
                ui,
                if valid_len { "VALID LEN" } else { "PAD NEEDED" },
                valid_len,
            );
            if frame.is_extended {
                draw_pdo_badge(ui, "EXT 29-bit", true);
            }
        });

        ui.label(
            RichText::new(format!(
                "ID: 0x{:X}  DLC_code: {}  Payload: {} bytes  BRS: {}",
                frame.can_id,
                dlc_code,
                fd_data.len(),
                frame.brs
            ))
            .monospace()
            .color(Color32::from_rgb(120, 230, 160)),
        );

        // FD DLC 映射表
        ui.add_space(4.0);
        egui::CollapsingHeader::new("CAN FD DLC 映射表").show(ui, |ui| {
            egui::Grid::new("fd_dlc_table")
                .num_columns(2)
                .spacing([12.0, 3.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label(RichText::new("DLC Code").strong());
                    ui.label(RichText::new("Payload Length").strong());
                    ui.end_row();
                    for dlc in 0..=15u8 {
                        let len = crate::models::canopen::fd_dlc_to_len(dlc);
                        let current = dlc_code == dlc;
                        let color = if current {
                            Color32::from_rgb(100, 220, 160)
                        } else {
                            Color32::from_rgb(180, 185, 195)
                        };
                        ui.label(RichText::new(format!("{}", dlc)).monospace().color(color));
                        ui.label(
                            RichText::new(format!("{} bytes", len))
                                .monospace()
                                .color(color),
                        );
                        ui.end_row();
                    }
                });
        });

        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            if ui.button("发送 CAN FD").clicked() {
                let mp = MultiProtocolFrame::can_fd_pdo(can_id as u16, &fd_data);
                let data = mp.to_bytes();
                send_canopen_frame(state, "CAN_FD", can_id as u16, &data);
            }
            if ui.button("复制帧").clicked() {
                ui.ctx().copy_text(format!(
                    "FD ID=0x{:X} DLC={} DATA={}",
                    frame.can_id,
                    dlc_code,
                    bytes_to_hex(&fd_data)
                ));
            }
        });
    });

    ui.add_space(8.0);

    // CANopen-over-FD: reuse standard tools with FD badge
    settings_card(ui, |ui| {
        ui.label(
            RichText::new("CANopen-over-FD 兼容模式")
                .strong()
                .size(14.0),
        );
        ui.label("CAN FD 帧可兼容标准 CANopen 协议（COB-ID ≤ 0x7FF, 数据 ≤ 8B 时自动降级为 CAN 2.0 语义）");
        ui.label("超过 8 字节的 FD 载荷可用于扩展 PDO 映射（如 CiA 1301 FD Profile）");
    });
}

// ═══════════════════════════════════════════════════════════════
// EtherCAT CoE 专用页面
// ═══════════════════════════════════════════════════════════════
fn show_ethercat_coe(ui: &mut Ui, state: &mut AppState) {
    settings_card(ui, |ui| {
        ui.label(
            RichText::new("EtherCAT CoE SDO 工具 / EtherCAT CoE SDO Tool")
                .strong()
                .size(15.0),
        );
        ui.add_space(6.0);

        ui.horizontal_wrapped(|ui| {
            ui.label("Slave Address:");
            ui.add(
                egui::TextEdit::singleline(&mut state.ui.canopen_node_id_text).desired_width(70.0),
            );
            ui.label("OD Index:");
            ui.add(
                egui::TextEdit::singleline(&mut state.ui.canopen_index_text).desired_width(90.0),
            );
            ui.label("Sub:");
            ui.add(
                egui::TextEdit::singleline(&mut state.ui.canopen_subidx_text).desired_width(60.0),
            );
        });

        ui.horizontal_wrapped(|ui| {
            ui.label("Payload:");
            ui.add(
                egui::TextEdit::singleline(&mut state.ui.canopen_payload_text)
                    .desired_width(210.0)
                    .hint_text("06 00"),
            );
            ui.checkbox(&mut state.ui.canopen_ecat_write, "Write (Download)");
        });

        let slave_addr = parse_u16_text(&state.ui.canopen_node_id_text).unwrap_or(1);
        let idx = parse_u16_text(&state.ui.canopen_index_text).unwrap_or(0x6040);
        let sub = parse_u8_text(&state.ui.canopen_subidx_text).unwrap_or(0);
        let payload = parse_hex_string(&state.ui.canopen_payload_text);

        let req = EcatCoeSdoRequest {
            slave_addr,
            index: idx,
            sub_index: sub,
            data: payload.clone(),
            is_write: state.ui.canopen_ecat_write,
        };
        let coe_frame = req.build_coe_frame();

        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            draw_pdo_badge(ui, "EtherCAT", true);
            draw_pdo_badge(ui, "CoE", true);
            draw_pdo_badge(
                ui,
                if state.ui.canopen_ecat_write {
                    "WRITE"
                } else {
                    "READ"
                },
                true,
            );
        });

        ui.label(
            RichText::new(&coe_frame.summary)
                .monospace()
                .color(Color32::from_rgb(120, 230, 160)),
        );
        ui.label(
            RichText::new(format!(
                "OD: {} (0x{:04X}:{:02X})",
                object_dict_name(idx, sub),
                idx,
                sub
            ))
            .color(Color32::from_rgb(130, 190, 255)),
        );

        // 帧内容预览
        ui.add_space(4.0);
        ui.label(
            RichText::new(format!(
                "MBX Header: {}  CoE Data: {}",
                bytes_to_hex(&coe_frame.mailbox_header),
                bytes_to_hex(&coe_frame.coe_data)
            ))
            .monospace()
            .size(10.5)
            .color(Color32::from_rgb(180, 190, 200)),
        );

        ui.horizontal_wrapped(|ui| {
            if ui.button("发送 CoE SDO").clicked() {
                let mp = MultiProtocolFrame::ecat_coe_sdo(
                    slave_addr,
                    idx,
                    sub,
                    &payload,
                    state.ui.canopen_ecat_write,
                );
                let data = mp.to_bytes();
                send_canopen_frame(state, "ECAT_CoE", slave_addr, &data);
            }
            if ui.button("复制帧").clicked() {
                let mut full = coe_frame.mailbox_header.clone();
                full.extend_from_slice(&coe_frame.coe_data);
                ui.ctx().copy_text(bytes_to_hex(&full));
            }
        });
    });

    ui.add_space(8.0);

    // EtherCAT CoE 帧分析器
    settings_card(ui, |ui| {
        ui.label(
            RichText::new("CoE 帧分析器 / CoE Frame Analyzer")
                .strong()
                .size(15.0),
        );
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            ui.label("Mailbox + CoE HEX:");
            ui.add(
                egui::TextEdit::singleline(&mut state.ui.canopen_ecat_analyze_hex)
                    .desired_width(500.0)
                    .hint_text("MBX header + CoE data"),
            );
        });

        let analyze_data = parse_hex_string(&state.ui.canopen_ecat_analyze_hex);
        if !analyze_data.is_empty() {
            let analysis = analyze_ecat_coe_frame(&analyze_data);
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                draw_pdo_badge(ui, "EtherCAT", true);
                draw_pdo_badge(
                    ui,
                    if analysis.valid { "VALID" } else { "INVALID" },
                    analysis.valid,
                );
            });
            ui.label(
                RichText::new(&analysis.summary)
                    .monospace()
                    .size(11.5)
                    .color(Color32::from_rgb(180, 220, 255)),
            );

            if !analysis.fields.is_empty() {
                let field_colors = [
                    Color32::from_rgb(100, 200, 255),
                    Color32::from_rgb(120, 230, 150),
                    Color32::from_rgb(255, 190, 100),
                    Color32::from_rgb(200, 150, 255),
                    Color32::from_rgb(255, 150, 150),
                ];
                egui::Grid::new("ecat_coe_fields")
                    .num_columns(4)
                    .spacing([10.0, 3.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label(RichText::new("■").strong());
                        ui.label(RichText::new("Field").strong());
                        ui.label(RichText::new("Raw").strong());
                        ui.label(RichText::new("Decoded").strong());
                        ui.end_row();
                        for f in &analysis.fields {
                            let color = field_colors[f.color_idx as usize % field_colors.len()];
                            ui.label(RichText::new("■").color(color));
                            ui.label(RichText::new(&f.name).size(10.5).color(color));
                            ui.label(RichText::new(&f.raw_hex).monospace().size(10.5));
                            ui.label(
                                RichText::new(&f.decoded)
                                    .size(10.5)
                                    .color(Color32::from_rgb(200, 210, 220)),
                            );
                            ui.end_row();
                        }
                    });
            }
        }
    });

    ui.add_space(8.0);

    // EtherCAT 状态机参考
    settings_card(ui, |ui| {
        ui.label(
            RichText::new("EtherCAT 状态机 / State Machine")
                .strong()
                .size(14.0),
        );
        ui.add_space(4.0);
        egui::Grid::new("ecat_states")
            .num_columns(3)
            .spacing([14.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label(RichText::new("Code").strong());
                ui.label(RichText::new("State").strong());
                ui.label(RichText::new("Description").strong());
                ui.end_row();
                for &code in &[1u8, 2, 3, 4, 8] {
                    ui.label(RichText::new(format!("0x{:02X}", code)).monospace());
                    ui.label(ecat_state_name(code));
                    let desc = match code {
                        1 => "初始化，无 PDO 交换",
                        2 => "可配置 SDO，无 PDI",
                        3 => "固件更新模式",
                        4 => "PDO 激活，输出安全值",
                        8 => "完全运行，实时 PDO 交换",
                        _ => "",
                    };
                    ui.label(desc);
                    ui.end_row();
                }
            });
    });

    // Shared OD browser
    ui.add_space(8.0);
    show_od_browser(ui, state);
}

// ═══════════════════════════════════════════════════════════════
// 标准 CANopen (CAN 2.0) 页面
// ═══════════════════════════════════════════════════════════════
fn show_canopen_standard(ui: &mut Ui, state: &mut AppState) {
    let node_id = parse_u8_text(&state.ui.canopen_node_id_text)
        .unwrap_or(1)
        .clamp(1, 127);
    let sdo_index = parse_u16_text(&state.ui.canopen_index_text).unwrap_or(0x1000);
    let sdo_subidx = parse_u8_text(&state.ui.canopen_subidx_text).unwrap_or(0);
    let heartbeat_ms = state
        .ui
        .canopen_heartbeat_ms_text
        .trim()
        .parse::<u16>()
        .unwrap_or(1000);

    settings_card(ui, |ui| {
        ui.label(
            RichText::new("网络总览 / Network Overview")
                .strong()
                .size(15.0),
        );
        ui.add_space(6.0);

        let node_ratio = node_id as f32 / 127.0;
        let can_entries = state
            .log_entries
            .iter()
            .filter(|e| e.channel.to_ascii_lowercase().contains("can"))
            .count();
        let health_ratio = if can_entries == 0 {
            1.0
        } else {
            (1.0 - (state.can.dropped_frames as f32 / can_entries as f32)).clamp(0.0, 1.0)
        };

        draw_meter(
            ui,
            "Node ID Density",
            node_ratio,
            format!("Node {} / 127", node_id),
            true,
        );
        draw_meter(
            ui,
            "Bus Health",
            health_ratio,
            format!("Dropped {} / {}", state.can.dropped_frames, can_entries),
            true,
        );
        draw_meter(
            ui,
            "Heartbeat Target",
            (heartbeat_ms as f32 / 5000.0).clamp(0.0, 1.0),
            format!("{} ms", heartbeat_ms),
            false,
        );

        ui.add_space(6.0);
        draw_id_map(ui, node_id);
    });

    ui.add_space(8.0);

    settings_card(ui, |ui| {
        ui.label(RichText::new("NMT 控制 / NMT Control").strong().size(15.0));
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            ui.label("Node ID:");
            ui.add(
                egui::TextEdit::singleline(&mut state.ui.canopen_node_id_text).desired_width(70.0),
            );

            let cmds = NmtCommand::all();
            state.ui.canopen_nmt_cmd_idx = state.ui.canopen_nmt_cmd_idx.min(cmds.len() - 1);
            let cmd = cmds[state.ui.canopen_nmt_cmd_idx];
            egui::ComboBox::from_id_salt("canopen_nmt_cmd")
                .selected_text(format!("{}", cmd))
                .width(300.0)
                .show_ui(ui, |ui| {
                    for (i, c) in cmds.iter().enumerate() {
                        ui.selectable_value(&mut state.ui.canopen_nmt_cmd_idx, i, format!("{}", c));
                    }
                });

            let frame = build_nmt(node_id, cmd);
            if ui.button("发送 NMT").clicked() {
                send_canopen_frame(state, "NMT", frame.cob_id, &frame.data);
            }
            if ui.button("复制").clicked() {
                ui.ctx().copy_text(format!(
                    "ID={:#05X} DATA={}",
                    frame.cob_id,
                    bytes_to_hex(&frame.data)
                ));
            }
        });

        let preview = build_nmt(node_id, NmtCommand::all()[state.ui.canopen_nmt_cmd_idx]);
        ui.label(
            RichText::new(format!(
                "COB-ID: {:#05X} ({})   DATA: {}",
                preview.cob_id,
                canopen_id_role(preview.cob_id),
                bytes_to_hex(&preview.data)
            ))
            .monospace()
            .color(Color32::from_rgb(120, 230, 160)),
        );
    });

    ui.add_space(8.0);

    settings_card(ui, |ui| {
        ui.label(RichText::new("SDO 客户端 / SDO Client").strong().size(15.0));
        ui.add_space(8.0);

        ui.horizontal_wrapped(|ui| {
            let actions = SdoAction::all();
            state.ui.canopen_sdo_action_idx =
                state.ui.canopen_sdo_action_idx.min(actions.len() - 1);
            let action = actions[state.ui.canopen_sdo_action_idx];

            egui::ComboBox::from_id_salt("canopen_sdo_action")
                .selected_text(format!("{}", action))
                .width(260.0)
                .show_ui(ui, |ui| {
                    for (i, a) in actions.iter().enumerate() {
                        ui.selectable_value(
                            &mut state.ui.canopen_sdo_action_idx,
                            i,
                            format!("{}", a),
                        );
                    }
                });

            ui.label("Index:");
            ui.add(
                egui::TextEdit::singleline(&mut state.ui.canopen_index_text).desired_width(90.0),
            );
            ui.label("Sub:");
            ui.add(
                egui::TextEdit::singleline(&mut state.ui.canopen_subidx_text).desired_width(60.0),
            );
            ui.label("Payload:");
            ui.add(
                egui::TextEdit::singleline(&mut state.ui.canopen_payload_text)
                    .desired_width(210.0)
                    .hint_text("11 22 33 44"),
            );
        });

        let payload = parse_hex_string(&state.ui.canopen_payload_text);
        let req = CanopenSdoRequest {
            node_id,
            action: SdoAction::all()[state.ui.canopen_sdo_action_idx],
            index: sdo_index,
            sub_index: sdo_subidx,
            payload,
        };
        let frame = req.build();

        ui.add_space(6.0);
        draw_sdo_bits(ui, frame.data[0]);
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new(format!(
                    "COB-ID: {:#05X} ({})   DATA: {}",
                    frame.cob_id,
                    canopen_id_role(frame.cob_id),
                    bytes_to_hex(&frame.data)
                ))
                .monospace()
                .color(Color32::from_rgb(120, 230, 160)),
            );
        });
        ui.label(
            RichText::new(format!(
                "OD: {} ({:#06X}:{:#04X})",
                object_dict_name(sdo_index, sdo_subidx),
                sdo_index,
                sdo_subidx
            ))
            .color(Color32::from_rgb(130, 190, 255)),
        );

        ui.horizontal_wrapped(|ui| {
            if ui.button("发送 SDO").clicked() {
                send_canopen_frame(state, "SDO", frame.cob_id, &frame.data);
            }
            if ui.button("复制 SDO").clicked() {
                ui.ctx().copy_text(format!(
                    "ID={:#05X} DATA={}",
                    frame.cob_id,
                    bytes_to_hex(&frame.data)
                ));
            }
        });
    });

    ui.add_space(8.0);

    settings_card(ui, |ui| {
        ui.label(
            RichText::new("PDO / Heartbeat / EMCY 工具")
                .strong()
                .size(15.0),
        );
        ui.add_space(8.0);

        egui::Grid::new("canopen_pdo_grid")
            .num_columns(2)
            .spacing([16.0, 8.0])
            .show(ui, |ui| {
                ui.label("PDO COB-ID:");
                ui.add(
                    egui::TextEdit::singleline(&mut state.ui.canopen_pdo_cobid_text)
                        .desired_width(100.0),
                );
                ui.end_row();

                ui.label("PDO Data:");
                ui.add(
                    egui::TextEdit::singleline(&mut state.ui.canopen_pdo_data_text)
                        .desired_width(320.0)
                        .hint_text("01 02 03 04 05 06 07 08"),
                );
                ui.end_row();

                ui.label("Heartbeat(ms):");
                ui.add(
                    egui::TextEdit::singleline(&mut state.ui.canopen_heartbeat_ms_text)
                        .desired_width(100.0),
                );
                ui.end_row();
            });

        let pdo_cob_id = parse_u16_text(&state.ui.canopen_pdo_cobid_text).unwrap_or(0x180);
        let pdo_data = parse_hex_string(&state.ui.canopen_pdo_data_text);
        let pdo = build_pdo(pdo_cob_id, &pdo_data);
        let hb = build_heartbeat_producer_sdo(node_id, heartbeat_ms);

        draw_meter(
            ui,
            "PDO Payload Utilization",
            (pdo.data.len() as f32 / 8.0).clamp(0.0, 1.0),
            format!("{} / 8 bytes", pdo.data.len()),
            true,
        );

        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new(format!(
                    "PDO => ID {:#05X} ({}) DATA {}",
                    pdo.cob_id,
                    canopen_id_role(pdo.cob_id),
                    bytes_to_hex(&pdo.data)
                ))
                .monospace(),
            );
        });
        ui.horizontal_wrapped(|ui| {
            if ui.button("发送 PDO").clicked() {
                send_canopen_frame(state, "PDO", pdo.cob_id, &pdo.data);
            }
            if ui.button("发送 Heartbeat 配置(SDO 0x1017)").clicked() {
                send_canopen_frame(state, "HB_CFG", hb.cob_id, &hb.data);
            }
        });

        ui.separator();
        ui.label(RichText::new("EMCY / Heartbeat 解码").strong());
        ui.horizontal_wrapped(|ui| {
            ui.label("Decode HEX:");
            ui.add(
                egui::TextEdit::singleline(&mut state.ui.canopen_decode_input)
                    .desired_width(360.0)
                    .hint_text("80 00 01 00 00 00 00 00"),
            );
        });

        let decode_bytes = parse_hex_string(&state.ui.canopen_decode_input);
        if let Some((err, reg, class_name)) = decode_emcy(&decode_bytes) {
            draw_status_chip(
                ui,
                &format!("EMCY: 0x{:04X} [{}] ERR_REG=0x{:02X}", err, class_name, reg),
                false,
            );
        }
        if let Some(state_code) = decode_bytes.first() {
            draw_status_chip(
                ui,
                &format!(
                    "Heartbeat State: {} (0x{:02X})",
                    decode_heartbeat_state(*state_code),
                    state_code
                ),
                true,
            );
        }
    });

    ui.add_space(8.0);

    // ═══════════════════════════════════════════════════════════════
    // PDO 映射管理器
    // ═══════════════════════════════════════════════════════════════
    settings_card(ui, |ui| {
        ui.label(
            RichText::new("PDO 映射管理器 / PDO Mapping Manager")
                .strong()
                .size(15.0),
        );
        ui.add_space(6.0);

        ui.horizontal_wrapped(|ui| {
            if ui.button("加载 CiA 402 预设").clicked() {
                state.canopen_pdo_configs = preset_pdo_configs();
                state
                    .canopen_log
                    .push("[INFO] Loaded CiA 402 preset PDO configs".into());
            }
            if ui.button("新增 PDO").clicked() {
                let pdo_count = state.canopen_pdo_configs.len();
                let new_pdo = PdoConfig {
                    name: format!("PDO{}", pdo_count + 1),
                    node_id,
                    cob_id: if pdo_count.is_multiple_of(2) {
                        0x180 + node_id as u16 + (pdo_count as u16 / 2) * 0x100
                    } else {
                        0x200 + node_id as u16 + (pdo_count as u16 / 2) * 0x100
                    },
                    ..PdoConfig::default()
                };
                state.canopen_pdo_configs.push(new_pdo);
            }
            if ui.button("导出 JSON").clicked() {
                if let Ok(json) = serde_json::to_string_pretty(&state.canopen_pdo_configs) {
                    ui.ctx().copy_text(json.clone());
                    state.status_message = "PDO configs exported to clipboard".into();
                    state
                        .canopen_log
                        .push("[INFO] PDO configs exported to clipboard".into());
                }
            }
            if ui.button("从剪贴板导入 JSON").clicked() {
                // 使用 decode_input 字段临时存放
                if let Ok(configs) =
                    serde_json::from_str::<Vec<PdoConfig>>(&state.ui.canopen_decode_input)
                {
                    let count = configs.len();
                    state.canopen_pdo_configs = configs;
                    state.status_message = format!("Imported {} PDO configs", count);
                    state
                        .canopen_log
                        .push(format!("[INFO] Imported {} PDO configs from input", count));
                } else if let Some(config) = PdoConfig::from_json(&state.ui.canopen_decode_input) {
                    state.canopen_pdo_configs.push(config);
                    state.status_message = "Imported 1 PDO config".into();
                }
            }
        });

        ui.add_space(6.0);

        // PDO 列表
        let mut remove_idx = None;
        let mut send_commands: Vec<(String, u16, Vec<u8>)> = Vec::new();
        let pdo_count = state.canopen_pdo_configs.len();

        for pdo_idx in 0..pdo_count {
            let pdo = &state.canopen_pdo_configs[pdo_idx];
            let total_bits = pdo.total_bits();
            let total_bytes = pdo.total_bytes();
            let dir_color = if pdo.direction == PdoDirection::Transmit {
                Color32::from_rgb(100, 200, 255)
            } else {
                Color32::from_rgb(120, 230, 150)
            };
            let pdo_name = pdo.name.clone();
            let pdo_cob_id = pdo.cob_id;
            let pdo_node_id = pdo.node_id;
            let pdo_enabled = pdo.enabled;
            let mappings_empty = pdo.mappings.is_empty();
            let mappings_count = pdo.mappings.len();

            egui::Frame::NONE
                .fill(Color32::from_rgb(22, 28, 38))
                .stroke(egui::Stroke::new(1.0, Color32::from_rgb(50, 60, 75)))
                .corner_radius(6.0)
                .inner_margin(egui::Margin::symmetric(10, 6))
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(
                            RichText::new(format!(
                                "{}  ",
                                state.canopen_pdo_configs[pdo_idx].direction
                            ))
                            .color(dir_color)
                            .strong(),
                        );
                        ui.label(RichText::new(&pdo_name).strong().size(12.5));
                        ui.separator();
                        ui.label(
                            RichText::new(format!("COB-ID: 0x{:03X}", pdo_cob_id))
                                .monospace()
                                .color(Color32::from_rgb(200, 210, 220)),
                        );
                        ui.label(RichText::new(format!("Node: {}", pdo_node_id)).monospace());
                        ui.label(
                            RichText::new(format!("{}b / {}B", total_bits, total_bytes))
                                .monospace(),
                        );
                        let valid = total_bytes <= 8;
                        draw_pdo_badge(ui, if valid { "OK" } else { "OVERFLOW" }, valid);
                        if pdo_enabled {
                            draw_pdo_badge(ui, "ENABLED", true);
                        } else {
                            draw_pdo_badge(ui, "DISABLED", false);
                        }
                    });

                    // 映射条目表
                    if !mappings_empty {
                        egui::Grid::new(format!("pdo_mapping_grid_{}", pdo_idx))
                            .num_columns(5)
                            .spacing([10.0, 3.0])
                            .striped(true)
                            .show(ui, |ui| {
                                ui.label(RichText::new("Signal").strong().size(10.5));
                                ui.label(RichText::new("Index:Sub").strong().size(10.5));
                                ui.label(RichText::new("Type").strong().size(10.5));
                                ui.label(RichText::new("Bits").strong().size(10.5));
                                ui.label(RichText::new("BitOffset").strong().size(10.5));
                                ui.end_row();

                                let mut bit_off = 0u32;
                                for m in &state.canopen_pdo_configs[pdo_idx].mappings {
                                    ui.label(
                                        RichText::new(&m.name)
                                            .size(10.5)
                                            .color(Color32::from_rgb(180, 220, 255)),
                                    );
                                    ui.label(
                                        RichText::new(format!(
                                            "0x{:04X}:{:02X}",
                                            m.index, m.sub_index
                                        ))
                                        .monospace()
                                        .size(10.5),
                                    );
                                    ui.label(RichText::new(format!("{}", m.data_type)).size(10.5));
                                    ui.label(
                                        RichText::new(format!("{}", m.bit_length))
                                            .monospace()
                                            .size(10.5),
                                    );
                                    ui.label(
                                        RichText::new(format!("{}", bit_off))
                                            .monospace()
                                            .size(10.5)
                                            .color(Color32::from_rgb(140, 150, 160)),
                                    );
                                    ui.end_row();
                                    bit_off += m.bit_length as u32;
                                }
                            });
                    }

                    ui.horizontal_wrapped(|ui| {
                        if ui.small_button("发送").clicked() {
                            let values: Vec<f64> = (0..mappings_count).map(|_| 0.0).collect();
                            let frame =
                                state.canopen_pdo_configs[pdo_idx].build_from_values(&values);
                            send_commands.push((
                                format!("PDO/{}", pdo_name),
                                frame.cob_id,
                                frame.data,
                            ));
                        }
                        if ui.small_button("复制帧").clicked() {
                            let values: Vec<f64> = (0..mappings_count).map(|_| 0.0).collect();
                            let frame =
                                state.canopen_pdo_configs[pdo_idx].build_from_values(&values);
                            ui.ctx().copy_text(format!(
                                "ID=0x{:03X} DATA={}",
                                frame.cob_id,
                                bytes_to_hex(&frame.data)
                            ));
                        }
                        if ui.small_button("删除").clicked() {
                            remove_idx = Some(pdo_idx);
                        }
                    });
                });
            ui.add_space(3.0);
        }

        // 执行延迟的发送命令
        for (tag, cob_id, data) in send_commands {
            send_canopen_frame(state, &tag, cob_id, &data);
        }

        if let Some(idx) = remove_idx {
            if idx < state.canopen_pdo_configs.len() {
                state.canopen_pdo_configs.remove(idx);
            }
        }

        // PDO 段可视化
        if !state.canopen_pdo_configs.is_empty() {
            ui.add_space(6.0);
            ui.label(RichText::new("PDO Payload Map").strong().size(12.0));
            for pdo in &state.canopen_pdo_configs {
                if !pdo.enabled || pdo.mappings.is_empty() {
                    continue;
                }
                let total = pdo.total_bits().max(1) as f32;
                let desired = egui::vec2(ui.available_width().max(200.0), 12.0);
                let (rect, _) = ui.allocate_exact_size(desired, egui::Sense::hover());
                let painter = ui.painter();
                painter.rect_filled(rect, 2.0, Color32::from_rgb(30, 35, 42));
                let colors = [
                    Color32::from_rgb(80, 170, 230),
                    Color32::from_rgb(100, 210, 140),
                    Color32::from_rgb(240, 180, 90),
                    Color32::from_rgb(190, 140, 240),
                    Color32::from_rgb(240, 130, 130),
                    Color32::from_rgb(130, 210, 210),
                ];
                let mut x = rect.left();
                for (i, m) in pdo.mappings.iter().enumerate() {
                    let w = rect.width() * (m.bit_length as f32 / total);
                    if w > 1.0 {
                        let r = egui::Rect::from_min_size(
                            egui::pos2(x, rect.top()),
                            egui::vec2(w, rect.height()),
                        );
                        painter.rect_filled(r, 1.0, colors[i % colors.len()]);
                    }
                    x += w;
                }
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        RichText::new(format!("{} (0x{:03X})", pdo.name, pdo.cob_id))
                            .size(10.0)
                            .color(Color32::from_rgb(160, 170, 180)),
                    );
                    for (i, m) in pdo.mappings.iter().enumerate() {
                        ui.label(RichText::new("■").color(colors[i % colors.len()]));
                        ui.label(RichText::new(format!("{} {}b", m.name, m.bit_length)).size(9.5));
                    }
                });
            }
        }
    });

    ui.add_space(8.0);

    // ═══════════════════════════════════════════════════════════════
    // PDO 数据解码器
    // ═══════════════════════════════════════════════════════════════
    if !state.canopen_pdo_configs.is_empty() {
        settings_card(ui, |ui| {
            ui.label(
                RichText::new("PDO 实时解码 / PDO Data Decoder")
                    .strong()
                    .size(15.0),
            );
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                ui.label("PDO Data HEX:");
                ui.add(
                    egui::TextEdit::singleline(&mut state.ui.canopen_pdo_decode_hex)
                        .desired_width(360.0)
                        .hint_text("例: E8 03 00 00 64 00"),
                );
            });

            let decode_data = parse_hex_string(&state.ui.canopen_pdo_decode_hex);
            if !decode_data.is_empty() {
                ui.add_space(4.0);
                for pdo in &state.canopen_pdo_configs {
                    if !pdo.enabled {
                        continue;
                    }
                    let values = pdo.decode_values(&decode_data);
                    if values.is_empty() {
                        continue;
                    }

                    egui::Frame::NONE
                        .fill(Color32::from_rgb(22, 30, 40))
                        .stroke(egui::Stroke::new(1.0, Color32::from_rgb(45, 55, 70)))
                        .corner_radius(4.0)
                        .inner_margin(egui::Margin::symmetric(8, 4))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(format!("{} (0x{:03X})", pdo.name, pdo.cob_id))
                                    .strong()
                                    .size(11.5),
                            );
                            egui::Grid::new(format!("pdo_decode_{}", pdo.cob_id))
                                .num_columns(3)
                                .spacing([12.0, 2.0])
                                .show(ui, |ui| {
                                    for (name, display, _raw) in &values {
                                        ui.label(
                                            RichText::new(name)
                                                .size(10.5)
                                                .color(Color32::from_rgb(140, 200, 255)),
                                        );
                                        ui.label(
                                            RichText::new(display)
                                                .monospace()
                                                .size(11.0)
                                                .color(Color32::from_rgb(120, 230, 160)),
                                        );
                                        ui.end_row();
                                    }
                                });
                        });
                    ui.add_space(3.0);
                }
            }
        });

        ui.add_space(8.0);
    }

    // ═══════════════════════════════════════════════════════════════
    // CANopen 帧深度分析器
    // ═══════════════════════════════════════════════════════════════
    settings_card(ui, |ui| {
        ui.label(
            RichText::new("CANopen 帧解析 / Frame Analyzer")
                .strong()
                .size(15.0),
        );
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            ui.label("COB-ID:");
            ui.add(
                egui::TextEdit::singleline(&mut state.ui.canopen_analyze_cobid_text)
                    .desired_width(80.0)
                    .hint_text("0x605"),
            );
            ui.label("Data HEX:");
            ui.add(
                egui::TextEdit::singleline(&mut state.ui.canopen_analyze_data_text)
                    .desired_width(320.0)
                    .hint_text("40 00 10 01 00 00 00 00"),
            );
        });

        let analyze_cob_id = parse_u16_text(&state.ui.canopen_analyze_cobid_text).unwrap_or(0);
        let analyze_data = parse_hex_string(&state.ui.canopen_analyze_data_text);

        if analyze_cob_id > 0 && !analyze_data.is_empty() {
            let analysis = analyze_canopen_frame(analyze_cob_id, &analyze_data);
            ui.add_space(4.0);

            // 状态 badge
            ui.horizontal_wrapped(|ui| {
                draw_pdo_badge(ui, analysis.role, true);
                draw_pdo_badge(ui, &format!("Node {}", analysis.node_id), true);
                draw_pdo_badge(
                    ui,
                    if analysis.valid { "VALID" } else { "INVALID" },
                    analysis.valid,
                );
            });

            ui.label(
                RichText::new(&analysis.summary)
                    .monospace()
                    .size(11.5)
                    .color(Color32::from_rgb(180, 220, 255)),
            );

            // 着色 HEX 地图
            if !analysis.fields.is_empty() {
                let field_colors = [
                    Color32::from_rgb(100, 200, 255),
                    Color32::from_rgb(120, 230, 150),
                    Color32::from_rgb(255, 190, 100),
                    Color32::from_rgb(200, 150, 255),
                    Color32::from_rgb(255, 150, 150),
                ];

                ui.add_space(2.0);
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        RichText::new("HEX│")
                            .monospace()
                            .size(10.5)
                            .color(Color32::from_rgb(80, 90, 100)),
                    );
                    for (i, b) in analyze_data.iter().enumerate() {
                        let color = analysis
                            .fields
                            .iter()
                            .find(|f| i >= f.offset && i < f.offset + f.length)
                            .map(|f| field_colors[f.color_idx as usize % field_colors.len()])
                            .unwrap_or(Color32::from_rgb(140, 145, 155));
                        ui.label(
                            RichText::new(format!("{:02X}", b))
                                .monospace()
                                .size(11.5)
                                .color(color),
                        );
                    }
                });

                // Fields table
                egui::Grid::new("canopen_analyze_fields")
                    .num_columns(4)
                    .spacing([10.0, 3.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label(RichText::new("■").strong());
                        ui.label(RichText::new("Field").strong());
                        ui.label(RichText::new("Raw").strong());
                        ui.label(RichText::new("Decoded").strong());
                        ui.end_row();

                        for f in &analysis.fields {
                            let color = field_colors[f.color_idx as usize % field_colors.len()];
                            ui.label(RichText::new("■").color(color));
                            ui.label(RichText::new(&f.name).size(10.5).color(color));
                            ui.label(RichText::new(&f.raw_hex).monospace().size(10.5));
                            ui.label(
                                RichText::new(&f.decoded)
                                    .size(10.5)
                                    .color(Color32::from_rgb(200, 210, 220)),
                            );
                            ui.end_row();
                        }
                    });
            }
        }
    });

    ui.add_space(8.0);
    show_od_browser(ui, state);
    show_canopen_log(ui, state);
}

fn show_od_browser(ui: &mut Ui, state: &mut AppState) {
    let sdo_index = parse_u16_text(&state.ui.canopen_index_text).unwrap_or(0x1000);
    let sdo_subidx = parse_u8_text(&state.ui.canopen_subidx_text).unwrap_or(0);

    settings_card(ui, |ui| {
        ui.label(
            RichText::new("增强对象字典 / Enhanced Object Dictionary")
                .strong()
                .size(15.0),
        );
        ui.add_space(8.0);

        let rows: &[(u16, u8)] = &[
            (0x1000, 0x00),
            (0x1001, 0x00),
            (0x1005, 0x00),
            (0x1006, 0x00),
            (0x1014, 0x00),
            (0x1017, 0x00),
            (0x1018, 0x00),
            (0x1018, 0x01),
            (0x1018, 0x02),
            (0x1018, 0x03),
            (0x1018, 0x04),
            (0x1400, 0x01),
            (0x1600, 0x00),
            (0x1800, 0x00),
            (0x1800, 0x01),
            (0x1800, 0x02),
            (0x1800, 0x05),
            (0x1A00, 0x00),
            (0x6040, 0x00),
            (0x6041, 0x00),
            (0x6060, 0x00),
            (0x6061, 0x00),
            (0x6064, 0x00),
            (0x606C, 0x00),
            (0x607A, 0x00),
        ];

        egui::Grid::new("canopen_od_grid")
            .num_columns(5)
            .spacing([14.0, 5.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label(RichText::new("Index").strong());
                ui.label(RichText::new("Sub").strong());
                ui.label(RichText::new("Name").strong());
                ui.label(RichText::new("Category").strong());
                ui.label(RichText::new("Target").strong());
                ui.end_row();

                for &(idx, sub) in rows {
                    let selected = idx == sdo_index && sub == sdo_subidx;
                    let category = if idx < 0x1000 {
                        "Reserved"
                    } else if idx < 0x2000 {
                        "Communication"
                    } else if idx < 0x6000 {
                        "Manufacturer"
                    } else {
                        "Device Profile"
                    };
                    let cat_color = match category {
                        "Communication" => Color32::from_rgb(100, 180, 255),
                        "Device Profile" => Color32::from_rgb(120, 230, 150),
                        _ => Color32::from_rgb(200, 200, 200),
                    };

                    ui.label(RichText::new(format!("0x{:04X}", idx)).monospace());
                    ui.label(RichText::new(format!("0x{:02X}", sub)).monospace());
                    ui.label(object_dict_name(idx, sub));
                    ui.label(RichText::new(category).size(10.5).color(cat_color));
                    if selected {
                        ui.colored_label(Color32::from_rgb(120, 220, 140), "◉ current");
                    } else {
                        ui.colored_label(Color32::GRAY, "○");
                    }
                    ui.end_row();
                }
            });
    });
}

fn show_canopen_log(ui: &mut Ui, state: &mut AppState) {
    if !state.canopen_log.is_empty() {
        ui.add_space(8.0);
        settings_card(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    RichText::new("CANopen 日志 / CANopen Log")
                        .strong()
                        .size(15.0),
                );
                if ui.button("清空").clicked() {
                    state.canopen_log.clear();
                }
            });
            ui.add_space(6.0);
            egui::ScrollArea::vertical()
                .max_height(150.0)
                .show(ui, |ui| {
                    for line in state.canopen_log.iter().rev().take(80) {
                        ui.label(
                            RichText::new(line)
                                .monospace()
                                .size(11.5)
                                .color(Color32::from_rgb(210, 220, 230)),
                        );
                    }
                });
        });
    }
}

fn send_canopen_frame(state: &mut AppState, tag: &str, cob_id: u16, data: &[u8]) {
    match state.send_data(data) {
        Ok(()) => {
            let line = format!("[TX {}] ID={:#05X} {}", tag, cob_id, bytes_to_hex(data));
            state.canopen_log.push(line.clone());
            state.status_message = format!("CANopen sent {} bytes", data.len());
            state.add_info_log(&line);
        }
        Err(e) => {
            state.status_message = format!("CANopen send error: {}", e);
            state.canopen_log.push(format!("[ERR {}] {}", tag, e));
        }
    }
}

fn draw_meter(ui: &mut Ui, name: &str, ratio: f32, text: String, high_good: bool) {
    let ratio = ratio.clamp(0.0, 1.0);
    let color = if high_good {
        if ratio > 0.75 {
            Color32::from_rgb(100, 220, 140)
        } else if ratio > 0.45 {
            Color32::from_rgb(255, 200, 100)
        } else {
            Color32::from_rgb(220, 120, 120)
        }
    } else if ratio < 0.35 {
        Color32::from_rgb(100, 220, 140)
    } else if ratio < 0.7 {
        Color32::from_rgb(255, 200, 100)
    } else {
        Color32::from_rgb(220, 120, 120)
    };

    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new(name).strong());
        ui.add(
            egui::ProgressBar::new(ratio)
                .desired_width(260.0)
                .fill(color)
                .text(text),
        );
    });
}

fn draw_status_chip(ui: &mut Ui, text: &str, ok: bool) {
    let (bg, fg) = if ok {
        (
            Color32::from_rgb(22, 58, 36),
            Color32::from_rgb(120, 230, 160),
        )
    } else {
        (
            Color32::from_rgb(65, 38, 20),
            Color32::from_rgb(255, 196, 110),
        )
    };

    egui::Frame::NONE
        .fill(bg)
        .stroke(egui::Stroke::new(1.0, fg.gamma_multiply(0.7)))
        .corner_radius(4.0)
        .inner_margin(egui::Margin::symmetric(8, 4))
        .show(ui, |ui| {
            ui.label(RichText::new(text).color(fg).monospace().size(11.5));
        });
}

fn draw_id_map(ui: &mut Ui, node_id: u8) {
    ui.label(RichText::new("CANopen COB-ID Map").strong().size(12.0));
    let rows = [
        ("NMT", 0x000u16),
        ("EMCY", 0x080u16 + node_id as u16),
        ("TPDO1", 0x180u16 + node_id as u16),
        ("RPDO1", 0x200u16 + node_id as u16),
        ("TSDO", 0x580u16 + node_id as u16),
        ("RSDO", 0x600u16 + node_id as u16),
        ("Heartbeat", 0x700u16 + node_id as u16),
    ];

    egui::Grid::new("canopen_id_map")
        .num_columns(3)
        .spacing([12.0, 4.0])
        .show(ui, |ui| {
            ui.label(RichText::new("Role").strong());
            ui.label(RichText::new("COB-ID").strong());
            ui.label(RichText::new("11-bit").strong());
            ui.end_row();
            for (role, id) in rows {
                ui.label(role);
                ui.label(RichText::new(format!("0x{:03X}", id)).monospace());
                ui.label(
                    RichText::new(format!("{:011b}", id & 0x07FF))
                        .monospace()
                        .color(Color32::from_rgb(140, 200, 255)),
                );
                ui.end_row();
            }
        });
}

fn draw_sdo_bits(ui: &mut Ui, cmd: u8) {
    ui.label(
        RichText::new("SDO Command Byte Bitfield")
            .strong()
            .size(12.0),
    );
    let bits = format!("{:08b}", cmd);
    ui.horizontal_wrapped(|ui| {
        ui.label(
            RichText::new(format!("0b{}", bits))
                .monospace()
                .size(12.5)
                .color(Color32::from_rgb(110, 190, 255)),
        );
        ui.separator();
        ui.label(format!("ccs/scs={}", cmd >> 5));
        ui.label(format!("n(bytes not used)={}", (cmd >> 2) & 0x03));
        ui.label(format!("e(expedited)={}", (cmd >> 1) & 0x01));
        ui.label(format!("s(size indicated)={}", cmd & 0x01));
    });
}

fn parse_u8_text(text: &str) -> Option<u8> {
    let t = text.trim();
    if let Some(hex) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        u8::from_str_radix(hex, 16).ok()
    } else {
        t.parse::<u8>().ok()
    }
}

fn parse_u16_text(text: &str) -> Option<u16> {
    let t = text.trim();
    if let Some(hex) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        u16::from_str_radix(hex, 16).ok()
    } else {
        t.parse::<u16>().ok()
    }
}

fn parse_u32_text(text: &str) -> Option<u32> {
    let t = text.trim();
    if let Some(hex) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        u32::from_str_radix(hex, 16).ok()
    } else {
        t.parse::<u32>().ok()
    }
}

fn draw_pdo_badge(ui: &mut Ui, text: &str, ok: bool) {
    let (bg, fg, stroke) = if ok {
        (
            Color32::from_rgb(24, 60, 36),
            Color32::from_rgb(120, 230, 150),
            Color32::from_rgb(60, 140, 80),
        )
    } else {
        (
            Color32::from_rgb(65, 42, 20),
            Color32::from_rgb(255, 195, 110),
            Color32::from_rgb(180, 120, 40),
        )
    };

    egui::Frame::NONE
        .fill(bg)
        .stroke(egui::Stroke::new(1.0, stroke))
        .corner_radius(4.0)
        .inner_margin(egui::Margin::symmetric(7, 2))
        .show(ui, |ui| {
            ui.label(RichText::new(text).color(fg).strong().size(10.0));
        });
}
