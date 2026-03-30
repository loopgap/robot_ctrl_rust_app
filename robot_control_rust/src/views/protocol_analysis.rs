use crate::app::{ActiveTab, AppState, DisplayMode, LogDirection};
use crate::models::canopen::canopen_id_role;
use crate::models::packet::parse_hex_string;
use crate::views::ui_kit::{page_header, settings_card};
use chrono::{NaiveTime, Timelike};
use egui::{self, Color32, RichText, Ui};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AnalyzerProtocol {
    Serial,
    Tcp,
    Udp,
    Can,
    CanFd,
    ModbusRtu,
    ModbusTcp,
    Usb,
}

impl AnalyzerProtocol {
    fn all() -> &'static [AnalyzerProtocol] {
        &[
            Self::Serial,
            Self::Tcp,
            Self::Udp,
            Self::Can,
            Self::CanFd,
            Self::ModbusRtu,
            Self::ModbusTcp,
            Self::Usb,
        ]
    }

    fn label(self) -> &'static str {
        match self {
            Self::Serial => "Serial",
            Self::Tcp => "TCP",
            Self::Udp => "UDP",
            Self::Can => "CAN 2.0",
            Self::CanFd => "CAN FD",
            Self::ModbusRtu => "Modbus RTU",
            Self::ModbusTcp => "Modbus TCP",
            Self::Usb => "USB",
        }
    }

    fn channel_match(self, channel: &str) -> bool {
        let c = channel.to_ascii_lowercase();
        match self {
            Self::Serial => c.contains("serial"),
            Self::Tcp => c.contains("tcp"),
            Self::Udp => c.contains("udp"),
            Self::Can => c.contains("can") && !c.contains("fd"),
            Self::CanFd => c.contains("can") && c.contains("fd"),
            Self::ModbusRtu => {
                c.contains("modbus rtu") || (c.contains("modbus") && c.contains("serial"))
            }
            Self::ModbusTcp => {
                c.contains("modbus tcp") || (c.contains("modbus") && c.contains("tcp"))
            }
            Self::Usb => c.contains("usb"),
        }
    }
}

struct AnalysisStats {
    frames: usize,
    tx: usize,
    rx: usize,
    info: usize,
    bytes: usize,
    avg_frame: f32,
    throughput_bps: f32,
}

struct IndustrialKpi {
    payload_utilization_pct: f32,
    frame_error_rate_pct: f32,
    bit_error_rate_ppm: f32,
    avg_inter_frame_ms: f32,
    jitter_ms: f32,
    duplicate_payload_ratio_pct: f32,
}

struct KpiTargets {
    payload_target_pct: f32,
    frame_error_max_pct: f32,
    ber_max_ppm: f32,
    ifg_max_ms: f32,
    jitter_max_ms: f32,
    duplicate_max_pct: f32,
}

struct DiagnosticCheck {
    name: String,
    pass: bool,
    detail: String,
}

/// Public CSV export for menu bar
pub fn export_analysis_csv(state: &AppState) -> anyhow::Result<std::path::PathBuf> {
    export_filtered_csv(&state.log_entries)
}

pub fn show(ui: &mut Ui, state: &mut AppState) {
    page_header(ui, "协议分析 / Protocol Analysis", "protocol_analysis");

    settings_card(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("该功能已集成在主界面（非独立窗口�?).strong());
            ui.separator();
            ui.label("��ǰҳ���Ǳ���/�ն˵��Թ���ͬһ����Ⱦ����");
        });
    });

    ui.add_space(8.0);

    let protocols = AnalyzerProtocol::all();
    state.ui.analysis_protocol_idx = state
        .ui
        .analysis_protocol_idx
        .min(protocols.len().saturating_sub(1));
    let protocol = protocols[state.ui.analysis_protocol_idx];

    settings_card(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("协议").strong());
            egui::ComboBox::from_id_salt("analysis_protocol_combo")
                .selected_text(protocol.label())
                .width(170.0)
                .show_ui(ui, |ui| {
                    for (idx, p) in protocols.iter().enumerate() {
                        ui.selectable_value(&mut state.ui.analysis_protocol_idx, idx, p.label());
                    }
                });

            ui.separator();
            ui.label(RichText::new("方向").strong());
            ui.checkbox(&mut state.ui.analysis_filter_tx, "TX");
            ui.checkbox(&mut state.ui.analysis_filter_rx, "RX");
            ui.checkbox(&mut state.ui.analysis_filter_info, "INFO");

            ui.separator();
            ui.label("关键�?");
            ui.add(egui::TextEdit::singleline(&mut state.ui.analysis_query).desired_width(180.0));
        });
    });

    let filtered = filtered_entries(state, protocol);
    let stats = calc_stats(&filtered);
    let kpi = calc_industrial_kpi(state, protocol, &filtered);
    let targets = protocol_kpi_targets(protocol);

    settings_card(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(format!("Frames: {}", stats.frames)).strong());
            ui.separator();
            ui.label(format!("TX: {}", stats.tx));
            ui.label(format!("RX: {}", stats.rx));
            ui.label(format!("INFO: {}", stats.info));
            ui.separator();
            ui.label(format!("Bytes: {}", stats.bytes));
            ui.label(format!("AvgFrame: {:.1} B", stats.avg_frame));
            ui.label(format!("Throughput: {:.1} B/s", stats.throughput_bps));
        });
    });

    settings_card(ui, |ui| {
        ui.label(
            RichText::new("Industrial KPI")
                .strong()
                .size(13.0)
                .color(Color32::from_rgb(0, 122, 204)),
        );
        ui.add_space(6.0);
        draw_metric_bar_high_good(
            ui,
            "Payload Utilization",
            kpi.payload_utilization_pct,
            targets.payload_target_pct,
            "%",
        );
        draw_metric_bar_low_good(
            ui,
            "Frame Error Rate",
            kpi.frame_error_rate_pct,
            targets.frame_error_max_pct,
            "%",
        );
        draw_metric_bar_low_good(
            ui,
            "BER(est)",
            kpi.bit_error_rate_ppm,
            targets.ber_max_ppm,
            " ppm",
        );
        draw_metric_bar_low_good(
            ui,
            "IFG(avg)",
            kpi.avg_inter_frame_ms,
            targets.ifg_max_ms,
            " ms",
        );
        draw_metric_bar_low_good(
            ui,
            "Jitter(std)",
            kpi.jitter_ms,
            targets.jitter_max_ms,
            " ms",
        );
        draw_metric_bar_low_good(
            ui,
            "Dup Payload",
            kpi.duplicate_payload_ratio_pct,
            targets.duplicate_max_pct,
            "%",
        );

        ui.add_space(2.0);
        let healthy = kpi.frame_error_rate_pct < targets.frame_error_max_pct
            && kpi.jitter_ms < targets.jitter_max_ms
            && kpi.duplicate_payload_ratio_pct < targets.duplicate_max_pct;
        ui.horizontal_wrapped(|ui| {
            draw_badge(ui, protocol.label(), true);
            draw_badge(
                ui,
                if healthy { "HEALTHY LINK" } else { "ATTENTION" },
                healthy,
            );
            draw_badge(ui, &format!("THR {:.1} B/s", stats.throughput_bps), true);
        });

        ui.add_space(4.0);
        for hint in anomaly_hints(protocol, &kpi, &targets) {
            ui.label(RichText::new(format!("�?{}", hint)).color(Color32::from_rgb(255, 210, 130)));
        }
    });

    ui.add_space(8.0);

    settings_card(ui, |ui| {
        ui.label(RichText::new("专业分析工具").strong());
        ui.add_space(6.0);

        ui.horizontal_wrapped(|ui| {
            if ui.button("载入最新过滤帧").clicked() {
                if let Some(entry) = filtered.last() {
                    state.ui.analysis_hex_input = entry
                        .data
                        .iter()
                        .map(|b| format!("{:02X}", b))
                        .collect::<Vec<_>>()
                        .join(" ");
                }
            }

            if ui.button("导出过滤日志 CSV").clicked() {
                match export_filtered_csv(&filtered) {
                    Ok(path) => {
                        state.status_message =
                            format!("Exported protocol logs: {}", path.display());
                        state.add_info_log(&state.status_message.clone());
                    }
                    Err(e) => {
                        state.report_error(format!("Export failed: {}", e));
                    }
                }
            }
        });

        ui.add_space(6.0);
        ui.label("HEX 帧输入（可手动粘贴用于离线分析）:");
        ui.add(
            egui::TextEdit::multiline(&mut state.ui.analysis_hex_input)
                .desired_rows(4)
                .desired_width(f32::INFINITY)
                .hint_text("例如: 01 03 00 00 00 0A C5 CD"),
        );

        let bytes = parse_hex_string(&state.ui.analysis_hex_input);
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            ui.label(format!("Len: {}", bytes.len()));
            ui.separator();
            ui.label(format!("XOR8: 0x{:02X}", xor8(&bytes)));
            ui.label(format!("SUM8: 0x{:02X}", sum8(&bytes)));
            ui.label(format!("CRC16(Modbus): 0x{:04X}", crc16_modbus(&bytes)));
            ui.label(format!("Entropy: {:.2}", entropy(&bytes)));
        });

        ui.add_space(8.0);
        draw_protocol_specific_tools(ui, protocol, &bytes);
        draw_protocol_diagnostics(ui, protocol, &bytes, &filtered, &kpi);

        ui.add_space(10.0);
        draw_frame_dissector(ui, protocol, &bytes);

        ui.add_space(10.0);
        draw_transaction_analysis(ui, protocol, &filtered);
    });

    ui.add_space(8.0);

    settings_card(ui, |ui| {
        ui.label(RichText::new("过滤结果预览").strong());
        ui.add_space(6.0);
        if filtered.is_empty() {
            ui.label(
                RichText::new("暂无匹配数据：可先连接设备并在终端调试页收发数据")
                    .color(Color32::from_rgb(255, 210, 130)),
            );
            ui.horizontal_wrapped(|ui| {
                if ui.button("前往连接管理").clicked() {
                    state.active_tab = ActiveTab::Connections;
                }
                if ui.button("前往终端调试").clicked() {
                    state.active_tab = ActiveTab::SerialDebug;
                }
                if ui.button("显示 INFO 日志").clicked() {
                    state.ui.analysis_filter_info = true;
                }
            });
            ui.add_space(6.0);
        }
        egui::ScrollArea::vertical()
            .max_height(240.0)
            .show(ui, |ui| {
                for entry in filtered.iter().rev().take(120) {
                    let tag = match entry.direction {
                        LogDirection::Tx => "TX",
                        LogDirection::Rx => "RX",
                        LogDirection::Info => "INFO",
                    };
                    let color = match entry.direction {
                        LogDirection::Tx => Color32::from_rgb(100, 200, 255),
                        LogDirection::Rx => Color32::from_rgb(120, 230, 120),
                        LogDirection::Info => Color32::from_rgb(240, 200, 120),
                    };
                    let data = format_data_with_mode(&entry.data, DisplayMode::Mixed);
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new(&entry.timestamp).small().color(Color32::GRAY));
                        ui.label(
                            RichText::new(format!("[{}]", entry.channel))
                                .small()
                                .color(Color32::LIGHT_GRAY),
                        );
                        ui.label(RichText::new(tag).color(color).strong());
                        ui.label(
                            RichText::new(data)
                                .monospace()
                                .color(Color32::from_rgb(220, 220, 220)),
                        );
                    });
                }
            });
    });
}

fn filtered_entries(state: &AppState, protocol: AnalyzerProtocol) -> Vec<crate::app::LogEntry> {
    let query = state.ui.analysis_query.trim().to_ascii_lowercase();

    state
        .log_entries
        .iter()
        .filter(|e| {
            let dir_ok = match e.direction {
                LogDirection::Tx => state.ui.analysis_filter_tx,
                LogDirection::Rx => state.ui.analysis_filter_rx,
                LogDirection::Info => state.ui.analysis_filter_info,
            };
            if !dir_ok {
                return false;
            }

            if !protocol.channel_match(&e.channel) {
                return false;
            }

            if query.is_empty() {
                return true;
            }

            let mixed = format_data_with_mode(&e.data, DisplayMode::Mixed).to_ascii_lowercase();
            mixed.contains(&query) || e.channel.to_ascii_lowercase().contains(&query)
        })
        .cloned()
        .collect()
}

fn calc_stats(entries: &[crate::app::LogEntry]) -> AnalysisStats {
    let mut tx = 0usize;
    let mut rx = 0usize;
    let mut info = 0usize;
    let mut bytes = 0usize;

    for e in entries {
        match e.direction {
            LogDirection::Tx => tx += 1,
            LogDirection::Rx => rx += 1,
            LogDirection::Info => info += 1,
        }
        bytes += e.data.len();
    }

    let avg_frame = if entries.is_empty() {
        0.0
    } else {
        bytes as f32 / entries.len() as f32
    };

    let throughput_bps = calc_recent_throughput(entries, 10.0);

    AnalysisStats {
        frames: entries.len(),
        tx,
        rx,
        info,
        bytes,
        avg_frame,
        throughput_bps,
    }
}

fn calc_recent_throughput(entries: &[crate::app::LogEntry], window_secs: f32) -> f32 {
    let mut samples: Vec<(NaiveTime, usize)> = Vec::new();
    for e in entries {
        if let Ok(t) = NaiveTime::parse_from_str(&e.timestamp, "%H:%M:%S%.3f") {
            samples.push((t, e.data.len()));
        }
    }
    if samples.len() < 2 {
        return 0.0;
    }
    samples.sort_by_key(|(t, _)| *t);
    let latest = samples.last().map(|v| v.0).unwrap_or_default();
    let min_t = latest - chrono::Duration::milliseconds_opt((window_secs * 1000.0) as i64).unwrap();
    let bytes: usize = samples
        .iter()
        .filter(|(t, _)| *t >= min_t)
        .map(|(_, n)| *n)
        .sum();
    bytes as f32 / window_secs.max(0.1)
}

fn calc_industrial_kpi(
    state: &AppState,
    protocol: AnalyzerProtocol,
    entries: &[crate::app::LogEntry],
) -> IndustrialKpi {
    let max_payload = protocol_payload_capacity(protocol) as f32;
    let total_bytes: usize = entries.iter().map(|e| e.data.len()).sum();
    let payload_utilization_pct = if entries.is_empty() || max_payload <= 0.0 {
        0.0
    } else {
        ((total_bytes as f32 / entries.len() as f32) / max_payload * 100.0).clamp(0.0, 100.0)
    };

    let error_events = protocol_error_count(state, protocol) as f32;
    let frame_error_rate_pct = if entries.is_empty() {
        0.0
    } else {
        (error_events / entries.len() as f32 * 100.0).max(0.0)
    };

    let total_bits = (total_bytes as f32 * 8.0).max(1.0);
    let bit_error_rate_ppm = (error_events / total_bits) * 1_000_000.0;

    let intervals = inter_frame_intervals_ms(entries);
    let avg_inter_frame_ms = if intervals.is_empty() {
        0.0
    } else {
        intervals.iter().sum::<f32>() / intervals.len() as f32
    };

    let jitter_ms = if intervals.len() < 2 {
        0.0
    } else {
        let mean = avg_inter_frame_ms;
        let var = intervals
            .iter()
            .map(|d| {
                let x = *d - mean;
                x * x
            })
            .sum::<f32>()
            / intervals.len() as f32;
        var.sqrt()
    };

    let duplicate_payload_ratio_pct = duplicate_payload_ratio(entries);

    IndustrialKpi {
        payload_utilization_pct,
        frame_error_rate_pct,
        bit_error_rate_ppm,
        avg_inter_frame_ms,
        jitter_ms,
        duplicate_payload_ratio_pct,
    }
}

fn protocol_payload_capacity(protocol: AnalyzerProtocol) -> usize {
    match protocol {
        AnalyzerProtocol::Serial => 256,
        AnalyzerProtocol::Tcp => 1460,
        AnalyzerProtocol::Udp => 1472,
        AnalyzerProtocol::Can => 8,
        AnalyzerProtocol::CanFd => 64,
        AnalyzerProtocol::ModbusRtu => 253,
        AnalyzerProtocol::ModbusTcp => 253,
        AnalyzerProtocol::Usb => 64,
    }
}

fn protocol_error_count(state: &AppState, protocol: AnalyzerProtocol) -> u64 {
    match protocol {
        AnalyzerProtocol::Serial => state.serial.error_count,
        AnalyzerProtocol::Tcp => state.tcp.error_count,
        AnalyzerProtocol::Udp => state.udp.error_count,
        AnalyzerProtocol::Can | AnalyzerProtocol::CanFd => state.can.dropped_frames,
        AnalyzerProtocol::ModbusRtu => state.serial.error_count,
        AnalyzerProtocol::ModbusTcp => state.tcp.error_count,
        AnalyzerProtocol::Usb => 0,
    }
}

fn inter_frame_intervals_ms(entries: &[crate::app::LogEntry]) -> Vec<f32> {
    let mut ts: Vec<u64> = entries
        .iter()
        .filter_map(|e| parse_ts_ms(&e.timestamp))
        .collect();
    if ts.len() < 2 {
        return Vec::new();
    }
    ts.sort_unstable();
    ts.windows(2)
        .filter_map(|w| {
            let delta = w[1].saturating_sub(w[0]) as f32;
            if delta > 0.0 {
                Some(delta)
            } else {
                None
            }
        })
        .collect()
}

fn parse_ts_ms(ts: &str) -> Option<u64> {
    let t = NaiveTime::parse_from_str(ts, "%H:%M:%S%.3f").ok()?;
    let secs = (t.hour() as u64 * 3600 + t.minute() as u64 * 60 + t.second() as u64) as u64;
    let ms = (t.nanosecond() as u64) / 1_000_000;
    Some(secs * 1000 + ms)
}

fn duplicate_payload_ratio(entries: &[crate::app::LogEntry]) -> f32 {
    if entries.is_empty() {
        return 0.0;
    }

    let mut map: HashMap<String, usize> = HashMap::new();
    for e in entries {
        let key = e
            .data
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join("");
        *map.entry(key).or_insert(0) += 1;
    }

    let duplicates: usize = map.values().map(|c| c.saturating_sub(1)).sum();
    duplicates as f32 / entries.len() as f32 * 100.0
}

fn anomaly_hints(
    protocol: AnalyzerProtocol,
    kpi: &IndustrialKpi,
    targets: &KpiTargets,
) -> Vec<String> {
    let mut hints = Vec::new();
    if kpi.frame_error_rate_pct > targets.frame_error_max_pct {
        hints.push("帧错误率偏高，建议检查线缆质量、终端匹配或电磁干扰�?.to_string());
    }
    if kpi.bit_error_rate_ppm > targets.ber_max_ppm {
        hints.push("估算 BER 较高，建议降低波特率并检查地线与屏蔽�?.to_string());
    }
    if kpi.jitter_ms > targets.jitter_max_ms {
        hints.push("帧间抖动较大，建议检查任务调度与缓冲区拥塞�?.to_string());
    }
    if kpi.duplicate_payload_ratio_pct > targets.duplicate_max_pct
        && matches!(
            protocol,
            AnalyzerProtocol::Tcp | AnalyzerProtocol::ModbusTcp
        )
    {
        hints.push("重复负载比例高，可能存在重传/应用层重复发送�?.to_string());
    }
    if hints.is_empty() {
        hints.push("当前链路指标正常，可继续进行功能级验证�?.to_string());
    }
    hints
}

fn protocol_kpi_targets(protocol: AnalyzerProtocol) -> KpiTargets {
    match protocol {
        AnalyzerProtocol::Serial => KpiTargets {
            payload_target_pct: 90.0,
            frame_error_max_pct: 2.0,
            ber_max_ppm: 500.0,
            ifg_max_ms: 80.0,
            jitter_max_ms: 20.0,
            duplicate_max_pct: 35.0,
        },
        AnalyzerProtocol::Tcp => KpiTargets {
            payload_target_pct: 92.0,
            frame_error_max_pct: 1.5,
            ber_max_ppm: 300.0,
            ifg_max_ms: 30.0,
            jitter_max_ms: 15.0,
            duplicate_max_pct: 30.0,
        },
        AnalyzerProtocol::Udp => KpiTargets {
            payload_target_pct: 92.0,
            frame_error_max_pct: 1.8,
            ber_max_ppm: 350.0,
            ifg_max_ms: 25.0,
            jitter_max_ms: 12.0,
            duplicate_max_pct: 25.0,
        },
        AnalyzerProtocol::Can => KpiTargets {
            payload_target_pct: 95.0,
            frame_error_max_pct: 1.0,
            ber_max_ppm: 200.0,
            ifg_max_ms: 10.0,
            jitter_max_ms: 8.0,
            duplicate_max_pct: 20.0,
        },
        AnalyzerProtocol::CanFd => KpiTargets {
            payload_target_pct: 95.0,
            frame_error_max_pct: 1.0,
            ber_max_ppm: 200.0,
            ifg_max_ms: 10.0,
            jitter_max_ms: 8.0,
            duplicate_max_pct: 20.0,
        },
        AnalyzerProtocol::ModbusRtu => KpiTargets {
            payload_target_pct: 88.0,
            frame_error_max_pct: 1.8,
            ber_max_ppm: 400.0,
            ifg_max_ms: 40.0,
            jitter_max_ms: 15.0,
            duplicate_max_pct: 25.0,
        },
        AnalyzerProtocol::ModbusTcp => KpiTargets {
            payload_target_pct: 90.0,
            frame_error_max_pct: 1.5,
            ber_max_ppm: 300.0,
            ifg_max_ms: 35.0,
            jitter_max_ms: 15.0,
            duplicate_max_pct: 30.0,
        },
        AnalyzerProtocol::Usb => KpiTargets {
            payload_target_pct: 85.0,
            frame_error_max_pct: 2.0,
            ber_max_ppm: 600.0,
            ifg_max_ms: 50.0,
            jitter_max_ms: 20.0,
            duplicate_max_pct: 30.0,
        },
    }
}

fn draw_badge(ui: &mut Ui, text: &str, ok: bool) {
    let (bg, fg, stroke) = if ok {
        (
            Color32::from_rgb(24, 64, 36),
            Color32::from_rgb(120, 230, 150),
            Color32::from_rgb(60, 150, 90),
        )
    } else {
        (
            Color32::from_rgb(68, 48, 22),
            Color32::from_rgb(255, 200, 120),
            Color32::from_rgb(190, 130, 50),
        )
    };

    egui::Frame::NONE
        .fill(bg)
        .stroke(egui::Stroke::new(1.0, stroke))
        .corner_radius(4.0)
        .inner_margin(egui::Margin::symmetric(8, 3))
        .show(ui, |ui| {
            ui.label(RichText::new(text).color(fg).strong().size(10.5));
        });
}

fn draw_metric_bar_high_good(ui: &mut Ui, name: &str, value: f32, max: f32, unit: &str) {
    let max = max.max(0.0001);
    let ratio = (value / max).clamp(0.0, 1.0);
    let color = if ratio >= 0.75 {
        Color32::from_rgb(80, 200, 120)
    } else if ratio >= 0.45 {
        Color32::from_rgb(255, 190, 90)
    } else {
        Color32::from_rgb(220, 120, 120)
    };
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new(name).strong());
        ui.add(
            egui::ProgressBar::new(ratio)
                .desired_width(220.0)
                .fill(color)
                .text(format!("{:.1}{}", value, unit)),
        );
    });
}

fn draw_metric_bar_low_good(ui: &mut Ui, name: &str, value: f32, max: f32, unit: &str) {
    let max = max.max(0.0001);
    let ratio = (value / max).clamp(0.0, 1.0);
    let color = if ratio <= 0.3 {
        Color32::from_rgb(80, 200, 120)
    } else if ratio <= 0.6 {
        Color32::from_rgb(255, 190, 90)
    } else {
        Color32::from_rgb(220, 120, 120)
    };
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new(name).strong());
        ui.add(
            egui::ProgressBar::new(ratio)
                .desired_width(220.0)
                .fill(color)
                .text(format!("{:.2}{}", value, unit)),
        );
    });
}

fn draw_segment_bar(ui: &mut Ui, title: &str, segments: &[(&str, f32, Color32)]) {
    ui.label(RichText::new(title).strong().size(11.5));
    let desired = egui::vec2(ui.available_width().max(200.0), 14.0);
    let (rect, _) = ui.allocate_exact_size(desired, egui::Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, 3.0, Color32::from_rgb(26, 30, 36));

    let total: f32 = segments.iter().map(|(_, v, _)| *v).sum::<f32>().max(0.0001);
    let mut x = rect.left();
    for (_, v, color) in segments {
        let w = rect.width() * (*v / total).clamp(0.0, 1.0);
        if w <= 0.5 {
            continue;
        }
        let r = egui::Rect::from_min_size(egui::pos2(x, rect.top()), egui::vec2(w, rect.height()));
        painter.rect_filled(r, 2.0, *color);
        x += w;
    }

    ui.add_space(3.0);
    ui.horizontal_wrapped(|ui| {
        for (name, v, color) in segments {
            ui.colored_label(*color, "�?);
            ui.label(format!("{} {:.0}%", name, *v));
            ui.add_space(6.0);
        }
    });
}

fn draw_protocol_specific_tools(ui: &mut Ui, protocol: AnalyzerProtocol, bytes: &[u8]) {
    ui.label(
        RichText::new("协议专用可视化分�?)
            .strong()
            .size(13.0)
            .color(Color32::from_rgb(0, 122, 204)),
    );
    ui.add_space(6.0);

    let payload_cap = protocol_payload_capacity(protocol).max(1) as f32;
    let payload_pct = (bytes.len() as f32 / payload_cap * 100.0).clamp(0.0, 100.0);

    egui::Frame::group(ui.style())
        .fill(ui.visuals().faint_bg_color)
        .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
        .corner_radius(8.0)
        .inner_margin(egui::Margin::symmetric(12, 10))
        .show(ui, |ui| {
            draw_metric_bar_high_good(ui, "Payload Rate", payload_pct, 100.0, "%");
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                draw_badge(ui, &format!("LEN {}B", bytes.len()), !bytes.is_empty());
                draw_badge(ui, &format!("CAP {}B", payload_cap as usize), true);
                draw_badge(
                    ui,
                    if payload_pct > 95.0 {
                        "Near Saturation"
                    } else {
                        "Within Capacity"
                    },
                    payload_pct <= 95.0,
                );
            });
        });

    ui.add_space(6.0);

    match protocol {
        AnalyzerProtocol::Serial => {
            let printable = bytes
                .iter()
                .filter(|b| b.is_ascii_graphic() || **b == b' ')
                .count();
            let ratio = if bytes.is_empty() {
                0.0
            } else {
                printable as f32 / bytes.len() as f32 * 100.0
            };
            let has_cr = bytes.contains(&b'\r');
            let has_lf = bytes.contains(&b'\n');
            egui::Frame::group(ui.style())
                .fill(ui.visuals().faint_bg_color)
                .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
                .corner_radius(8.0)
                .inner_margin(egui::Margin::symmetric(12, 10))
                .show(ui, |ui| {
                    ui.label(RichText::new("Serial Content Diagnostics").strong());
                    draw_metric_bar_high_good(ui, "Printable Ratio", ratio, 100.0, "%");
                    draw_metric_bar_low_good(ui, "Entropy", entropy(bytes), 8.0, " bit");
                    ui.horizontal_wrapped(|ui| {
                        draw_badge(ui, "CR", has_cr);
                        draw_badge(ui, "LF", has_lf);
                        draw_badge(ui, "FRAME HINT", has_cr || has_lf);
                    });
                });
        }
        AnalyzerProtocol::Tcp => {
            let mtu_pressure = (bytes.len() as f32 / 1460.0 * 100.0).clamp(0.0, 100.0);
            let frag_risk = bytes.len() > 1400;
            egui::Frame::group(ui.style())
                .fill(ui.visuals().faint_bg_color)
                .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
                .corner_radius(8.0)
                .inner_margin(egui::Margin::symmetric(12, 10))
                .show(ui, |ui| {
                    ui.label(RichText::new("TCP Session Pressure").strong());
                    draw_metric_bar_low_good(ui, "MTU Headroom", mtu_pressure, 100.0, "%");
                    draw_metric_bar_low_good(
                        ui,
                        "Dup Risk(instant)",
                        if frag_risk { 70.0 } else { 20.0 },
                        100.0,
                        "%",
                    );
                    ui.horizontal_wrapped(|ui| {
                        draw_badge(ui, "MSS 1460", true);
                        draw_badge(
                            ui,
                            if frag_risk {
                                "FRAGMENT RISK"
                            } else {
                                "FRAGMENT OK"
                            },
                            !frag_risk,
                        );
                    });
                });
        }
        AnalyzerProtocol::Udp => {
            let datagram_pressure = (bytes.len() as f32 / 1472.0 * 100.0).clamp(0.0, 100.0);
            let seq = bytes.first().copied().unwrap_or(0);
            egui::Frame::group(ui.style())
                .fill(ui.visuals().faint_bg_color)
                .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
                .corner_radius(8.0)
                .inner_margin(egui::Margin::symmetric(12, 10))
                .show(ui, |ui| {
                    ui.label(RichText::new("UDP Datagram Analysis").strong());
                    draw_metric_bar_low_good(
                        ui,
                        "Datagram Pressure",
                        datagram_pressure,
                        100.0,
                        "%",
                    );
                    draw_metric_bar_low_good(
                        ui,
                        "Loss Sensitivity",
                        if bytes.len() < 16 { 60.0 } else { 28.0 },
                        100.0,
                        "%",
                    );
                    ui.horizontal_wrapped(|ui| {
                        draw_badge(ui, &format!("SEQ_HINT 0x{:02X}", seq), true);
                        draw_badge(ui, "NO RETRANSMIT", true);
                        draw_badge(
                            ui,
                            if datagram_pressure > 95.0 {
                                "OVERSIZE WARN"
                            } else {
                                "DATAGRAM OK"
                            },
                            datagram_pressure <= 95.0,
                        );
                    });
                });
        }
        AnalyzerProtocol::Can => {
            let dlc_ok = bytes.len() <= 8;
            let can_id = if bytes.len() >= 2 {
                (((bytes[0] as u16) << 3) | ((bytes[1] as u16) >> 5)) & 0x07FF
            } else {
                0
            };
            let id_pct = can_id as f32 / 0x07FF as f32 * 100.0;
            let zero_bytes = bytes.iter().filter(|b| **b == 0).count() as f32;
            let data_domain_ok = bytes.is_empty() || (zero_bytes / bytes.len() as f32) < 0.9;
            egui::Frame::group(ui.style())
                .fill(ui.visuals().faint_bg_color)
                .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
                .corner_radius(8.0)
                .inner_margin(egui::Margin::symmetric(12, 10))
                .show(ui, |ui| {
                    ui.label(RichText::new("CAN ID / Data Domain Validation").strong());
                    draw_metric_bar_high_good(ui, "ID Space Position", id_pct, 100.0, "%");
                    draw_metric_bar_low_good(
                        ui,
                        "Zero-byte Density",
                        if bytes.is_empty() {
                            0.0
                        } else {
                            zero_bytes / bytes.len() as f32 * 100.0
                        },
                        100.0,
                        "%",
                    );
                    ui.horizontal_wrapped(|ui| {
                        draw_badge(ui, &format!("ID 0x{:03X}", can_id), dlc_ok);
                        draw_badge(ui, &format!("DLC {}", bytes.len()), dlc_ok);
                        draw_badge(ui, "DATA DOMAIN", data_domain_ok);
                    });
                });
        }
        AnalyzerProtocol::CanFd => {
            let dlc_ok = is_can_fd_len_valid(bytes.len());
            let can_id = if bytes.len() >= 4 {
                (((bytes[0] as u32) << 21)
                    | ((bytes[1] as u32) << 13)
                    | ((bytes[2] as u32) << 5)
                    | ((bytes[3] as u32) >> 3))
                    & 0x1FFF_FFFF
            } else {
                0
            };
            egui::Frame::group(ui.style())
                .fill(ui.visuals().faint_bg_color)
                .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
                .corner_radius(8.0)
                .inner_margin(egui::Margin::symmetric(12, 10))
                .show(ui, |ui| {
                    ui.label(RichText::new("CAN FD Domain + DLC Map").strong());
                    draw_metric_bar_high_good(
                        ui,
                        "FD Payload Rate",
                        (bytes.len() as f32 / 64.0) * 100.0,
                        100.0,
                        "%",
                    );
                    draw_segment_bar(
                        ui,
                        "DLC Validity Map",
                        &[
                            (
                                "Valid",
                                if dlc_ok { 85.0 } else { 20.0 },
                                Color32::from_rgb(90, 200, 130),
                            ),
                            (
                                "Invalid",
                                if dlc_ok { 15.0 } else { 80.0 },
                                Color32::from_rgb(220, 120, 120),
                            ),
                        ],
                    );
                    ui.horizontal_wrapped(|ui| {
                        draw_badge(ui, &format!("ID 0x{:X}", can_id), true);
                        draw_badge(ui, &format!("LEN {}", bytes.len()), dlc_ok);
                        draw_badge(ui, "CAN-FD MAP", dlc_ok);
                    });
                });
        }
        AnalyzerProtocol::ModbusRtu => {
            if bytes.len() >= 4 {
                let slave = bytes[0];
                let fc = bytes[1];
                let (payload, crc_ok) = {
                    let payload = &bytes[..bytes.len() - 2];
                    let crc_given =
                        u16::from_le_bytes([bytes[bytes.len() - 2], bytes[bytes.len() - 1]]);
                    let crc_calc = crc16_modbus(payload);
                    (payload.len(), crc_given == crc_calc)
                };
                egui::Frame::group(ui.style())
                    .fill(ui.visuals().faint_bg_color)
                    .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
                    .corner_radius(8.0)
                    .inner_margin(egui::Margin::symmetric(12, 10))
                    .show(ui, |ui| {
                        ui.label(RichText::new("Modbus RTU Frame Integrity").strong());
                        draw_metric_bar_high_good(
                            ui,
                            "Payload Ratio",
                            payload as f32 / 253.0 * 100.0,
                            100.0,
                            "%",
                        );
                        draw_metric_bar_low_good(
                            ui,
                            "Frame Noise(Entropy)",
                            entropy(bytes),
                            8.0,
                            " bit",
                        );
                        ui.horizontal_wrapped(|ui| {
                            draw_badge(ui, &format!("SLAVE {}", slave), true);
                            draw_badge(
                                ui,
                                &format!("FC 0x{:02X} {}", fc, modbus_fc_name(fc)),
                                (fc & 0x80) == 0,
                            );
                            draw_badge(ui, if crc_ok { "CRC16 OK" } else { "CRC16 BAD" }, crc_ok);
                        });
                    });
            } else {
                ui.label("Modbus RTU: need at least 4 bytes");
            }
        }
        AnalyzerProtocol::ModbusTcp => {
            if bytes.len() >= 8 {
                let tx_id = u16::from_be_bytes([bytes[0], bytes[1]]);
                let proto = u16::from_be_bytes([bytes[2], bytes[3]]);
                let len = u16::from_be_bytes([bytes[4], bytes[5]]);
                let unit = bytes[6];
                let func = bytes[7];
                let proto_ok = proto == 0;
                let expected_total = 6 + len as usize;
                let len_ok = bytes.len() >= expected_total;
                egui::Frame::group(ui.style())
                    .fill(ui.visuals().faint_bg_color)
                    .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
                    .corner_radius(8.0)
                    .inner_margin(egui::Margin::symmetric(12, 10))
                    .show(ui, |ui| {
                        ui.label(RichText::new("Modbus TCP MBAP Validation").strong());
                        draw_metric_bar_low_good(
                            ui,
                            "Length Delta",
                            expected_total.saturating_sub(bytes.len()) as f32,
                            16.0,
                            " B",
                        );
                        draw_segment_bar(
                            ui,
                            "MBAP Health",
                            &[
                                (
                                    "ProtocolID",
                                    if proto_ok { 50.0 } else { 20.0 },
                                    Color32::from_rgb(90, 200, 130),
                                ),
                                (
                                    "Length",
                                    if len_ok { 50.0 } else { 80.0 },
                                    if len_ok {
                                        Color32::from_rgb(90, 200, 130)
                                    } else {
                                        Color32::from_rgb(220, 120, 120)
                                    },
                                ),
                            ],
                        );
                        ui.horizontal_wrapped(|ui| {
                            draw_badge(ui, &format!("TID {}", tx_id), true);
                            draw_badge(ui, &format!("UNIT {}", unit), true);
                            draw_badge(
                                ui,
                                &format!("FC 0x{:02X} {}", func, modbus_fc_name(func)),
                                (func & 0x80) == 0,
                            );
                        });
                    });
            } else {
                ui.label("Modbus TCP: need at least 8 bytes");
            }
        }
        AnalyzerProtocol::Usb => {
            if bytes.len() >= 8 {
                let bm = bytes[0];
                let req = bytes[1];
                let w_value = u16::from_le_bytes([bytes[2], bytes[3]]);
                let w_index = u16::from_le_bytes([bytes[4], bytes[5]]);
                let w_len = u16::from_le_bytes([bytes[6], bytes[7]]);
                let (dir, typ, recipient) = decode_usb_bm_request_type(bm);
                let class_hint = detect_usb_class(bm, req, w_value, w_index);
                let class_req_name = usb_class_request_name(class_hint, req);
                let txn_type = if w_len == 0 {
                    "Control-NoData"
                } else if dir == "Device->Host" {
                    "Control-IN"
                } else {
                    "Control-OUT"
                };
                let dir_host = if dir == "Host->Device" { 65.0 } else { 35.0 };
                let dir_dev = 100.0 - dir_host;
                let type_mix = match typ {
                    "Standard" => (70.0, 15.0, 15.0),
                    "Class" => (15.0, 70.0, 15.0),
                    "Vendor" => (10.0, 15.0, 75.0),
                    _ => (20.0, 20.0, 60.0),
                };

                egui::Frame::group(ui.style())
                    .fill(ui.visuals().faint_bg_color)
                    .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
                    .corner_radius(8.0)
                    .inner_margin(egui::Margin::symmetric(12, 10))
                    .show(ui, |ui| {
                        ui.label(RichText::new("USB Transaction Analyzer").strong());
                        ui.add_space(2.0);
                        // Class identification badge
                        ui.horizontal_wrapped(|ui| {
                            let class_color = class_hint.color();
                            ui.label(
                                RichText::new(format!("�?{} Class", class_hint.label()))
                                    .strong()
                                    .color(class_color),
                            );
                            ui.label(
                                RichText::new(format!("�?{} �?{}", class_req_name, txn_type))
                                    .monospace()
                                    .size(11.0)
                                    .color(Color32::from_rgb(200, 200, 215)),
                            );
                        });
                        ui.add_space(2.0);
                        draw_metric_bar_high_good(ui, "Setup Payload", w_len as f32, 1024.0, " B");
                        draw_segment_bar(
                            ui,
                            "Direction Split",
                            &[
                                ("Host->Dev", dir_host, Color32::from_rgb(110, 180, 255)),
                                ("Dev->Host", dir_dev, Color32::from_rgb(120, 220, 140)),
                            ],
                        );
                        draw_segment_bar(
                            ui,
                            "Request Type Distribution",
                            &[
                                ("Standard", type_mix.0, Color32::from_rgb(120, 220, 140)),
                                ("Class", type_mix.1, Color32::from_rgb(255, 190, 90)),
                                ("Vendor/Other", type_mix.2, Color32::from_rgb(220, 120, 120)),
                            ],
                        );
                        ui.horizontal_wrapped(|ui| {
                            draw_badge(ui, txn_type, true);
                            draw_badge(ui, &format!("{} / {}", typ, recipient), true);
                            draw_badge(
                                ui,
                                &format!("REQ 0x{:02X} {}", req, class_req_name),
                                true,
                            );
                        });

                        // Class-specific detail panel
                        match class_hint {
                            UsbClassHint::CdcAcm => {
                                ui.add_space(4.0);
                                ui.label(
                                    RichText::new("CDC ACM Detail")
                                        .strong()
                                        .color(Color32::from_rgb(100, 220, 160)),
                                );
                                if (req == 0x20 || req == 0x21) && bytes.len() >= 15 {
                                    let baud = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
                                    let stop = match bytes[12] { 0 => "1", 1 => "1.5", 2 => "2", _ => "?" };
                                    let parity = match bytes[13] { 0 => "N", 1 => "O", 2 => "E", 3 => "M", 4 => "S", _ => "?" };
                                    let data_b = bytes[14];
                                    ui.label(
                                        RichText::new(format!(
                                            "  Line Coding: {} baud, {}{}1 (data={}, stop={}, parity={})",
                                            baud, data_b, parity, data_b, stop, parity
                                        ))
                                        .monospace()
                                        .size(11.0)
                                        .color(Color32::from_rgb(180, 220, 180)),
                                    );
                                } else if req == 0x22 {
                                    let dtr = (w_value & 0x01) != 0;
                                    let rts = (w_value & 0x02) != 0;
                                    ui.label(
                                        RichText::new(format!(
                                            "  Control Line State: DTR={} RTS={}",
                                            if dtr { "ON" } else { "OFF" },
                                            if rts { "ON" } else { "OFF" }
                                        ))
                                        .monospace()
                                        .size(11.0)
                                        .color(Color32::from_rgb(180, 220, 180)),
                                    );
                                }
                            }
                            UsbClassHint::Hid => {
                                ui.add_space(4.0);
                                ui.label(
                                    RichText::new("HID Detail")
                                        .strong()
                                        .color(Color32::from_rgb(255, 200, 100)),
                                );
                                let report_type = match (w_value >> 8) as u8 { 1 => "Input", 2 => "Output", 3 => "Feature", _ => "?" };
                                let report_id = (w_value & 0xFF) as u8;
                                ui.label(
                                    RichText::new(format!(
                                        "  Report Type: {}  ID: {}  Interface: {}",
                                        report_type, report_id, w_index
                                    ))
                                    .monospace()
                                    .size(11.0)
                                    .color(Color32::from_rgb(220, 200, 140)),
                                );
                            }
                            UsbClassHint::MassStorage => {
                                ui.add_space(4.0);
                                ui.label(
                                    RichText::new("Mass Storage Detail")
                                        .strong()
                                        .color(Color32::from_rgb(200, 160, 255)),
                                );
                                ui.label(
                                    RichText::new(format!(
                                        "  Request: {}  Interface: {}",
                                        usb_msc_request_name(req), w_index
                                    ))
                                    .monospace()
                                    .size(11.0)
                                    .color(Color32::from_rgb(200, 180, 240)),
                                );
                            }
                            _ => {}
                        }

                        ui.label(
                            RichText::new(format!(
                                "wValue=0x{:04X}  wIndex=0x{:04X}  bmRequestType=0x{:02X}",
                                w_value, w_index, bm
                            ))
                            .monospace()
                            .color(Color32::from_rgb(170, 170, 185)),
                        );
                    });
            } else if let Some(bot_type) = detect_usb_bot_frame(bytes) {
                // MSC BOT frame (not a setup packet)
                egui::Frame::group(ui.style())
                    .fill(ui.visuals().faint_bg_color)
                    .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
                    .corner_radius(8.0)
                    .inner_margin(egui::Margin::symmetric(12, 10))
                    .show(ui, |ui| {
                        ui.label(RichText::new("USB Mass Storage BOT Analyzer").strong());
                        ui.add_space(2.0);
                        match bot_type {
                            "CBW" if bytes.len() >= 31 => {
                                let scsi_op = bytes[15];
                                let xfer_len =
                                    u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
                                let flags = bytes[12];
                                ui.horizontal_wrapped(|ui| {
                                    draw_badge(ui, "CBW", true);
                                    draw_badge(
                                        ui,
                                        &format!(
                                            "SCSI 0x{:02X} {}",
                                            scsi_op,
                                            scsi_opcode_name(scsi_op)
                                        ),
                                        true,
                                    );
                                    draw_badge(
                                        ui,
                                        if flags & 0x80 != 0 {
                                            "Data-IN"
                                        } else {
                                            "Data-OUT"
                                        },
                                        true,
                                    );
                                    draw_badge(ui, &format!("{} B", xfer_len), xfer_len < 1048576);
                                });
                            }
                            "CSW" if bytes.len() >= 13 => {
                                let status = bytes[12];
                                let residue =
                                    u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
                                let status_name = match status {
                                    0 => "Passed",
                                    1 => "Failed",
                                    2 => "Phase Error",
                                    _ => "Reserved",
                                };
                                ui.horizontal_wrapped(|ui| {
                                    draw_badge(ui, "CSW", true);
                                    draw_badge(ui, status_name, status == 0);
                                    draw_badge(ui, &format!("Residue {} B", residue), residue == 0);
                                });
                            }
                            _ => {
                                ui.label("BOT frame: insufficient data");
                            }
                        }
                    });
            } else {
                ui.label("USB: setup packet analysis requires at least 8 bytes");
            }
        }
    }
}

fn draw_protocol_diagnostics(
    ui: &mut Ui,
    protocol: AnalyzerProtocol,
    bytes: &[u8],
    entries: &[crate::app::LogEntry],
    kpi: &IndustrialKpi,
) {
    let checks = protocol_diagnostic_checks(protocol, bytes, entries, kpi);
    ui.add_space(10.0);
    ui.label(
        RichText::new("工业级诊断清�?)
            .strong()
            .size(13.0)
            .color(Color32::from_rgb(0, 122, 204)),
    );
    ui.add_space(4.0);

    for c in &checks {
        let (border_color, icon, icon_color, bg) = if c.pass {
            (
                Color32::from_rgb(50, 140, 80),
                "�?PASS",
                Color32::from_rgb(100, 220, 140),
                Color32::from_rgb(20, 45, 30),
            )
        } else {
            (
                Color32::from_rgb(180, 120, 40),
                "�?WARN",
                Color32::from_rgb(255, 190, 100),
                Color32::from_rgb(50, 38, 18),
            )
        };
        egui::Frame::NONE
            .fill(bg)
            .stroke(egui::Stroke::new(1.5, border_color))
            .corner_radius(5.0)
            .inner_margin(egui::Margin::symmetric(10, 6))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label(RichText::new(icon).color(icon_color).strong().size(11.5));
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(&c.name)
                            .strong()
                            .size(12.0)
                            .color(Color32::from_rgb(220, 220, 230)),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new(&c.detail)
                                .monospace()
                                .size(11.0)
                                .color(Color32::from_rgb(160, 160, 175)),
                        );
                    });
                });
            });
        ui.add_space(2.0);
    }

    let pass_count = checks.iter().filter(|c| c.pass).count();
    let total = checks.len();
    let all_pass = pass_count == total;
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        let frac = if total > 0 {
            pass_count as f32 / total as f32
        } else {
            1.0
        };
        let color = if all_pass {
            Color32::from_rgb(80, 200, 120)
        } else {
            Color32::from_rgb(255, 190, 80)
        };
        ui.add(
            egui::ProgressBar::new(frac)
                .desired_width(200.0)
                .fill(color)
                .text(RichText::new(format!("{}/{} Passed", pass_count, total)).size(11.0)),
        );
        if all_pass {
            draw_badge(ui, "ALL PASS", true);
        }
    });
}

fn protocol_diagnostic_checks(
    protocol: AnalyzerProtocol,
    bytes: &[u8],
    entries: &[crate::app::LogEntry],
    kpi: &IndustrialKpi,
) -> Vec<DiagnosticCheck> {
    match protocol {
        AnalyzerProtocol::Serial => serial_checks(bytes, kpi),
        AnalyzerProtocol::Tcp => tcp_checks(bytes, entries, kpi),
        AnalyzerProtocol::Udp => udp_checks(bytes, entries, kpi),
        AnalyzerProtocol::Can => can_checks(bytes, kpi),
        AnalyzerProtocol::CanFd => can_fd_checks(bytes, kpi),
        AnalyzerProtocol::ModbusRtu => modbus_rtu_checks(bytes, kpi),
        AnalyzerProtocol::ModbusTcp => modbus_tcp_checks(bytes, kpi),
        AnalyzerProtocol::Usb => usb_checks(bytes, kpi),
    }
}

fn serial_checks(bytes: &[u8], kpi: &IndustrialKpi) -> Vec<DiagnosticCheck> {
    let printable = bytes
        .iter()
        .filter(|b| b.is_ascii_graphic() || **b == b' ')
        .count();
    let printable_ratio = if bytes.is_empty() {
        0.0
    } else {
        printable as f32 / bytes.len() as f32 * 100.0
    };
    let has_line_end = bytes.contains(&b'\r') || bytes.contains(&b'\n');
    vec![
        DiagnosticCheck {
            name: "Payload sanity".into(),
            pass: !bytes.is_empty(),
            detail: format!("payload_len={}B", bytes.len()),
        },
        DiagnosticCheck {
            name: "Printable ratio".into(),
            pass: printable_ratio >= 30.0 || bytes.is_empty(),
            detail: format!("printable={:.1}%", printable_ratio),
        },
        DiagnosticCheck {
            name: "Frame boundary hint".into(),
            pass: has_line_end || bytes.is_empty(),
            detail: format!("CR/LF detected={}", has_line_end),
        },
        DiagnosticCheck {
            name: "Timing jitter".into(),
            pass: kpi.jitter_ms <= 20.0,
            detail: format!("jitter={:.2}ms", kpi.jitter_ms),
        },
    ]
}

fn tcp_checks(
    bytes: &[u8],
    entries: &[crate::app::LogEntry],
    kpi: &IndustrialKpi,
) -> Vec<DiagnosticCheck> {
    let mtu_ok = bytes.len() <= 1460;
    let dup_ok = kpi.duplicate_payload_ratio_pct < 35.0;
    let retrans_hint = duplicate_payload_ratio(entries);
    vec![
        DiagnosticCheck {
            name: "MTU fragmentation risk".into(),
            pass: mtu_ok,
            detail: format!("payload={}B (limit=1460B)", bytes.len()),
        },
        DiagnosticCheck {
            name: "Duplicate payload".into(),
            pass: dup_ok,
            detail: format!("dup_ratio={:.1}%", retrans_hint),
        },
        DiagnosticCheck {
            name: "Frame error rate".into(),
            pass: kpi.frame_error_rate_pct < 2.0,
            detail: format!("fer={:.3}%", kpi.frame_error_rate_pct),
        },
        DiagnosticCheck {
            name: "Inter-frame jitter".into(),
            pass: kpi.jitter_ms < 20.0,
            detail: format!("jitter={:.2}ms", kpi.jitter_ms),
        },
    ]
}

fn udp_checks(
    bytes: &[u8],
    entries: &[crate::app::LogEntry],
    kpi: &IndustrialKpi,
) -> Vec<DiagnosticCheck> {
    let mtu_ok = bytes.len() <= 1472;
    let out_of_order = infer_sequence_disorder(entries);
    vec![
        DiagnosticCheck {
            name: "Datagram size".into(),
            pass: mtu_ok,
            detail: format!("payload={}B (limit=1472B)", bytes.len()),
        },
        DiagnosticCheck {
            name: "Sequence continuity (heuristic)".into(),
            pass: !out_of_order,
            detail: format!("sequence_disorder={}", out_of_order),
        },
        DiagnosticCheck {
            name: "Drop/error pressure".into(),
            pass: kpi.frame_error_rate_pct < 2.0,
            detail: format!("fer={:.3}%", kpi.frame_error_rate_pct),
        },
        DiagnosticCheck {
            name: "Jitter".into(),
            pass: kpi.jitter_ms < 20.0,
            detail: format!("jitter={:.2}ms", kpi.jitter_ms),
        },
    ]
}

fn can_checks(bytes: &[u8], kpi: &IndustrialKpi) -> Vec<DiagnosticCheck> {
    let dlc_ok = bytes.len() <= 8;
    vec![
        DiagnosticCheck {
            name: "DLC range".into(),
            pass: dlc_ok,
            detail: format!("dlc={}", bytes.len()),
        },
        DiagnosticCheck {
            name: "Payload utilization".into(),
            pass: kpi.payload_utilization_pct <= 100.0,
            detail: format!("util={:.1}%", kpi.payload_utilization_pct),
        },
        DiagnosticCheck {
            name: "Frame error".into(),
            pass: kpi.frame_error_rate_pct < 2.0,
            detail: format!("fer={:.3}%", kpi.frame_error_rate_pct),
        },
        DiagnosticCheck {
            name: "Bus jitter".into(),
            pass: kpi.jitter_ms < 10.0,
            detail: format!("jitter={:.2}ms", kpi.jitter_ms),
        },
    ]
}

fn can_fd_checks(bytes: &[u8], kpi: &IndustrialKpi) -> Vec<DiagnosticCheck> {
    let dlc_ok = is_can_fd_len_valid(bytes.len());
    vec![
        DiagnosticCheck {
            name: "CAN FD DLC map".into(),
            pass: dlc_ok,
            detail: format!("len={} (valid: 0..8/12/16/20/24/32/48/64)", bytes.len()),
        },
        DiagnosticCheck {
            name: "Payload utilization".into(),
            pass: kpi.payload_utilization_pct <= 100.0,
            detail: format!("util={:.1}%", kpi.payload_utilization_pct),
        },
        DiagnosticCheck {
            name: "Frame error".into(),
            pass: kpi.frame_error_rate_pct < 2.0,
            detail: format!("fer={:.3}%", kpi.frame_error_rate_pct),
        },
        DiagnosticCheck {
            name: "Timing jitter".into(),
            pass: kpi.jitter_ms < 10.0,
            detail: format!("jitter={:.2}ms", kpi.jitter_ms),
        },
    ]
}

fn modbus_rtu_checks(bytes: &[u8], kpi: &IndustrialKpi) -> Vec<DiagnosticCheck> {
    let frame_ok = bytes.len() >= 4;
    let crc_ok = if bytes.len() >= 4 {
        let payload = &bytes[..bytes.len() - 2];
        let crc_given = u16::from_le_bytes([bytes[bytes.len() - 2], bytes[bytes.len() - 1]]);
        crc16_modbus(payload) == crc_given
    } else {
        false
    };
    let exception_resp = bytes.len() >= 2 && (bytes[1] & 0x80) != 0;

    vec![
        DiagnosticCheck {
            name: "Frame length".into(),
            pass: frame_ok,
            detail: format!("len={}", bytes.len()),
        },
        DiagnosticCheck {
            name: "CRC16 validation".into(),
            pass: crc_ok || !frame_ok,
            detail: format!("crc_ok={}", crc_ok),
        },
        DiagnosticCheck {
            name: "Exception response".into(),
            pass: !exception_resp,
            detail: format!("exception_bit={}", exception_resp),
        },
        DiagnosticCheck {
            name: "Error pressure".into(),
            pass: kpi.frame_error_rate_pct < 2.0,
            detail: format!("fer={:.3}%", kpi.frame_error_rate_pct),
        },
    ]
}

fn modbus_tcp_checks(bytes: &[u8], kpi: &IndustrialKpi) -> Vec<DiagnosticCheck> {
    let mbap_ok = bytes.len() >= 8;
    let (proto_zero, len_ok, exception_resp) = if bytes.len() >= 8 {
        let proto = u16::from_be_bytes([bytes[2], bytes[3]]);
        let pdu_len = u16::from_be_bytes([bytes[4], bytes[5]]) as usize;
        let expected_min_total = 6 + pdu_len;
        let func = bytes[7];
        (
            proto == 0,
            bytes.len() >= expected_min_total,
            (func & 0x80) != 0,
        )
    } else {
        (false, false, false)
    };

    vec![
        DiagnosticCheck {
            name: "MBAP header".into(),
            pass: mbap_ok,
            detail: format!("len={} (>=8 required)", bytes.len()),
        },
        DiagnosticCheck {
            name: "Protocol ID".into(),
            pass: proto_zero || !mbap_ok,
            detail: format!("proto_id_zero={}", proto_zero),
        },
        DiagnosticCheck {
            name: "Length consistency".into(),
            pass: len_ok || !mbap_ok,
            detail: format!("mbap_length_ok={}", len_ok),
        },
        DiagnosticCheck {
            name: "Exception response".into(),
            pass: !exception_resp,
            detail: format!("exception_bit={}", exception_resp),
        },
        DiagnosticCheck {
            name: "Error pressure".into(),
            pass: kpi.frame_error_rate_pct < 2.0,
            detail: format!("fer={:.3}%", kpi.frame_error_rate_pct),
        },
    ]
}

fn usb_checks(bytes: &[u8], kpi: &IndustrialKpi) -> Vec<DiagnosticCheck> {
    let setup_ok = bytes.len() >= 8;
    let (dir, typ, recipient) = if setup_ok {
        decode_usb_bm_request_type(bytes[0])
    } else {
        ("N/A", "N/A", "N/A")
    };

    let mut checks = vec![
        DiagnosticCheck {
            name: "Setup packet size".into(),
            pass: setup_ok,
            detail: format!("len={} (>=8 required)", bytes.len()),
        },
        DiagnosticCheck {
            name: "bmRequestType decode".into(),
            pass: setup_ok,
            detail: format!("dir={} type={} recipient={}", dir, typ, recipient),
        },
        DiagnosticCheck {
            name: "Entropy/structure".into(),
            pass: entropy(bytes) < 7.8 || bytes.is_empty(),
            detail: format!("entropy={:.2}", entropy(bytes)),
        },
        DiagnosticCheck {
            name: "Transfer stability".into(),
            pass: kpi.jitter_ms < 20.0,
            detail: format!("jitter={:.2}ms", kpi.jitter_ms),
        },
    ];

    // Class-specific checks
    if setup_ok {
        let bm = bytes[0];
        let req = bytes[1];
        let w_value = u16::from_le_bytes([bytes[2], bytes[3]]);
        let w_index = u16::from_le_bytes([bytes[4], bytes[5]]);
        let w_len = u16::from_le_bytes([bytes[6], bytes[7]]);
        let class_hint = detect_usb_class(bm, req, w_value, w_index);

        checks.push(DiagnosticCheck {
            name: "USB Class".into(),
            pass: class_hint != UsbClassHint::Unknown,
            detail: format!("class={}", class_hint.label()),
        });

        match class_hint {
            UsbClassHint::CdcAcm => {
                // Check data stage for LINE_CODING requests
                let lc_ok = if req == 0x20 || req == 0x21 {
                    w_len == 7
                } else {
                    true
                };
                checks.push(DiagnosticCheck {
                    name: "CDC line coding len".into(),
                    pass: lc_ok,
                    detail: format!("wLength={} (expected 7 for LC)", w_len),
                });
                if req == 0x20 && bytes.len() >= 15 {
                    let baud = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
                    let baud_ok = matches!(
                        baud,
                        300 | 600
                            | 1200
                            | 2400
                            | 4800
                            | 9600
                            | 14400
                            | 19200
                            | 28800
                            | 38400
                            | 57600
                            | 76800
                            | 115200
                            | 230400
                            | 460800
                            | 500000
                            | 576000
                            | 921600
                            | 1000000
                            | 1500000
                            | 2000000
                            | 3000000
                    );
                    checks.push(DiagnosticCheck {
                        name: "Baud rate validity".into(),
                        pass: baud_ok || baud > 0,
                        detail: format!("baud={}", baud),
                    });
                }
            }
            UsbClassHint::Hid => {
                let report_type_ok = if req == 0x01 || req == 0x09 {
                    matches!((w_value >> 8) as u8, 1..=3)
                } else {
                    true
                };
                checks.push(DiagnosticCheck {
                    name: "HID report type".into(),
                    pass: report_type_ok,
                    detail: format!("report_type={}", (w_value >> 8) as u8),
                });
            }
            UsbClassHint::MassStorage => {
                let req_ok = matches!(req, 0xFE | 0xFF);
                checks.push(DiagnosticCheck {
                    name: "MSC request valid".into(),
                    pass: req_ok,
                    detail: format!("bRequest=0x{:02X} [{}]", req, usb_msc_request_name(req)),
                });
            }
            _ => {}
        }

        // Descriptor GET check
        if req == 0x06 && typ == "Standard" {
            let desc_type = (w_value >> 8) as u8;
            let desc_ok = matches!(desc_type, 1..=11 | 0x21..=0x30);
            checks.push(DiagnosticCheck {
                name: "Descriptor type valid".into(),
                pass: desc_ok,
                detail: format!(
                    "desc_type=0x{:02X} [{}]",
                    desc_type,
                    usb_descriptor_type_name(desc_type)
                ),
            });
        }

        // wLength sanity
        let w_len_ok = w_len <= 4096;
        checks.push(DiagnosticCheck {
            name: "wLength range".into(),
            pass: w_len_ok,
            detail: format!("wLength={} (<=4096)", w_len),
        });
    }

    // BOT frame checks
    if let Some(bot_type) = detect_usb_bot_frame(bytes) {
        match bot_type {
            "CBW" => {
                let sig_ok = bytes.len() >= 31;
                let lun_ok = sig_ok && bytes[13] <= 15;
                let cb_len_ok = sig_ok && (bytes[14] & 0x1F) <= 16;
                checks.push(DiagnosticCheck {
                    name: "CBW signature".into(),
                    pass: sig_ok,
                    detail: "0x55534243 [USBC]".into(),
                });
                checks.push(DiagnosticCheck {
                    name: "CBW LUN range".into(),
                    pass: lun_ok,
                    detail: format!("LUN={}", if sig_ok { bytes[13] & 0x0F } else { 0xFF }),
                });
                checks.push(DiagnosticCheck {
                    name: "CBW CB length".into(),
                    pass: cb_len_ok,
                    detail: format!("cbLen={}", if sig_ok { bytes[14] & 0x1F } else { 0xFF }),
                });
            }
            "CSW" => {
                let sig_ok = bytes.len() >= 13;
                let status_ok = sig_ok && bytes[12] <= 2;
                checks.push(DiagnosticCheck {
                    name: "CSW signature".into(),
                    pass: sig_ok,
                    detail: "0x55534253 [USBS]".into(),
                });
                checks.push(DiagnosticCheck {
                    name: "CSW status".into(),
                    pass: status_ok && bytes.get(12).copied().unwrap_or(0xFF) == 0,
                    detail: format!(
                        "status={}",
                        bytes
                            .get(12)
                            .map(|s| match s {
                                0 => "Passed",
                                1 => "Failed",
                                2 => "PhaseError",
                                _ => "?",
                            })
                            .unwrap_or("?")
                    ),
                });
            }
            _ => {}
        }
    }

    checks
}

fn is_can_fd_len_valid(len: usize) -> bool {
    matches!(len, 0..=8 | 12 | 16 | 20 | 24 | 32 | 48 | 64)
}

fn infer_sequence_disorder(entries: &[crate::app::LogEntry]) -> bool {
    let mut seqs = Vec::new();
    for e in entries {
        if let Some(first) = e.data.first() {
            seqs.push(*first);
        }
    }
    if seqs.len() < 3 {
        return false;
    }

    let mut disorder = 0usize;
    for w in seqs.windows(2) {
        let prev = w[0];
        let next = w[1];
        let expected = prev.wrapping_add(1);
        if next != expected {
            disorder += 1;
        }
    }
    disorder > seqs.len() / 3
}

fn decode_usb_bm_request_type(v: u8) -> (&'static str, &'static str, &'static str) {
    let dir = if (v & 0x80) != 0 {
        "Device->Host"
    } else {
        "Host->Device"
    };
    let typ = match (v >> 5) & 0x03 {
        0 => "Standard",
        1 => "Class",
        2 => "Vendor",
        _ => "Reserved",
    };
    let recipient = match v & 0x1F {
        0 => "Device",
        1 => "Interface",
        2 => "Endpoint",
        3 => "Other",
        _ => "Reserved",
    };
    (dir, typ, recipient)
}

fn modbus_fc_name(fc: u8) -> &'static str {
    match fc & 0x7F {
        0x01 => "Read Coils",
        0x02 => "Read Discrete Inputs",
        0x03 => "Read Holding Registers",
        0x04 => "Read Input Registers",
        0x05 => "Write Single Coil",
        0x06 => "Write Single Register",
        0x0F => "Write Multiple Coils",
        0x10 => "Write Multiple Registers",
        0x17 => "Read/Write Multiple Registers",
        _ => "Unknown",
    }
}

fn usb_request_name(req: u8) -> &'static str {
    match req {
        0x00 => "GET_STATUS",
        0x01 => "CLEAR_FEATURE",
        0x03 => "SET_FEATURE",
        0x05 => "SET_ADDRESS",
        0x06 => "GET_DESCRIPTOR",
        0x07 => "SET_DESCRIPTOR",
        0x08 => "GET_CONFIGURATION",
        0x09 => "SET_CONFIGURATION",
        0x0A => "GET_INTERFACE",
        0x0B => "SET_INTERFACE",
        0x0C => "SYNCH_FRAME",
        _ => "Vendor/Class-Specific",
    }
}

// ══════════════════════════════════════════════════════════════�?// USB 协议特化分析引擎 / USB Protocol Specialization Engine
// ══════════════════════════════════════════════════════════════�?
/// USB device class codes �?human-readable name
fn usb_class_name(class_code: u8) -> &'static str {
    match class_code {
        0x00 => "Device (use interface)",
        0x01 => "Audio",
        0x02 => "CDC (Comm)",
        0x03 => "HID",
        0x05 => "Physical",
        0x06 => "Image / Still Capture",
        0x07 => "Printer",
        0x08 => "Mass Storage",
        0x09 => "Hub",
        0x0A => "CDC-Data",
        0x0B => "Smart Card",
        0x0D => "Content Security",
        0x0E => "Video",
        0x0F => "Personal Healthcare",
        0x10 => "Audio/Video",
        0x11 => "Billboard",
        0xDC => "Diagnostic",
        0xE0 => "Wireless Controller",
        0xEF => "Miscellaneous",
        0xFE => "Application-Specific",
        0xFF => "Vendor-Specific",
        _ => "Unknown",
    }
}

/// USB descriptor type �?name
fn usb_descriptor_type_name(desc_type: u8) -> &'static str {
    match desc_type {
        1 => "DEVICE",
        2 => "CONFIGURATION",
        3 => "STRING",
        4 => "INTERFACE",
        5 => "ENDPOINT",
        6 => "DEVICE_QUALIFIER",
        7 => "OTHER_SPEED_CONFIG",
        8 => "INTERFACE_POWER",
        9 => "OTG",
        10 => "DEBUG",
        11 => "INTERFACE_ASSOCIATION",
        0x21 => "HID",
        0x22 => "HID_REPORT",
        0x23 => "HID_PHYSICAL",
        0x24 => "CS_INTERFACE",
        0x25 => "CS_ENDPOINT",
        0x29 => "HUB",
        0x2A => "SUPERSPEED_HUB",
        0x30 => "SS_ENDPOINT_COMPANION",
        _ => "UNKNOWN",
    }
}

/// USB PID (Packet ID) �?name
fn usb_pid_name(pid: u8) -> &'static str {
    match pid & 0x0F {
        0x01 => "OUT",
        0x09 => "IN",
        0x05 => "SOF",
        0x0D => "SETUP",
        0x03 => "DATA0",
        0x0B => "DATA1",
        0x07 => "DATA2",
        0x0F => "MDATA",
        0x02 => "ACK",
        0x0A => "NAK",
        0x0E => "STALL",
        0x06 => "NYET",
        0x0C => "PRE/ERR",
        0x08 => "SPLIT",
        0x04 => "PING",
        _ => "RESERVED",
    }
}

/// USB endpoint transfer type from bmAttributes
fn usb_transfer_type_name(bm_attrs: u8) -> &'static str {
    match bm_attrs & 0x03 {
        0 => "Control",
        1 => "Isochronous",
        2 => "Bulk",
        3 => "Interrupt",
        _ => unreachable!(),
    }
}

/// USB CDC class-specific request decoder
fn usb_cdc_request_name(b_request: u8) -> &'static str {
    match b_request {
        0x00 => "SEND_ENCAPSULATED_COMMAND",
        0x01 => "GET_ENCAPSULATED_RESPONSE",
        0x02 => "SET_COMM_FEATURE",
        0x03 => "GET_COMM_FEATURE",
        0x04 => "CLEAR_COMM_FEATURE",
        0x10 => "SET_AUX_LINE_STATE",
        0x11 => "SET_HOOK_STATE",
        0x12 => "PULSE_SETUP",
        0x13 => "SEND_PULSE",
        0x14 => "SET_PULSE_TIME",
        0x15 => "RING_AUX_JACK",
        0x20 => "SET_LINE_CODING",
        0x21 => "GET_LINE_CODING",
        0x22 => "SET_CONTROL_LINE_STATE",
        0x23 => "SEND_BREAK",
        0x30 => "SET_RINGER_PARMS",
        0x31 => "GET_RINGER_PARMS",
        0x32 => "SET_OPERATION_PARMS",
        0x33 => "GET_OPERATION_PARMS",
        0x34 => "SET_LINE_PARMS",
        0x35 => "GET_LINE_PARMS",
        0x36 => "DIAL_DIGITS",
        0x40 => "SET_UNIT_PARAMETER",
        0x41 => "GET_UNIT_PARAMETER",
        0x42 => "CLEAR_UNIT_PARAMETER",
        0x43 => "GET_PROFILE",
        0x44 => "SET_ETHERNET_MULTICAST_FILTERS",
        0x45 => "SET_ETHERNET_PM_PATTERN_FILTER",
        0x46 => "GET_ETHERNET_PM_PATTERN_FILTER",
        0x47 => "SET_ETHERNET_PACKET_FILTER",
        0x48 => "GET_ETHERNET_STATISTIC",
        0x50 => "SET_ATM_DATA_FORMAT",
        0x51 => "GET_ATM_DEVICE_STATISTICS",
        0x64 => "GET_NTB_PARAMETERS",
        0x65 => "GET_NET_ADDRESS",
        0x66 => "SET_NET_ADDRESS",
        0x67 => "GET_NTB_FORMAT",
        0x68 => "SET_NTB_FORMAT",
        0x69 => "GET_NTB_INPUT_SIZE",
        0x6A => "SET_NTB_INPUT_SIZE",
        0x6B => "GET_MAX_DATAGRAM_SIZE",
        0x6C => "SET_MAX_DATAGRAM_SIZE",
        0x6D => "GET_CRC_MODE",
        0x6E => "SET_CRC_MODE",
        _ => "CDC_UNKNOWN",
    }
}

/// USB HID class-specific request decoder
fn usb_hid_request_name(b_request: u8) -> &'static str {
    match b_request {
        0x01 => "GET_REPORT",
        0x02 => "GET_IDLE",
        0x03 => "GET_PROTOCOL",
        0x09 => "SET_REPORT",
        0x0A => "SET_IDLE",
        0x0B => "SET_PROTOCOL",
        _ => "HID_UNKNOWN",
    }
}

/// USB Mass Storage class-specific request decoder
fn usb_msc_request_name(b_request: u8) -> &'static str {
    match b_request {
        0xFE => "GET_MAX_LUN",
        0xFF => "BULK_ONLY_RESET",
        _ => "MSC_UNKNOWN",
    }
}

/// USB Audio class-specific request decoder
fn usb_audio_request_name(b_request: u8) -> &'static str {
    match b_request {
        0x01 => "SET_CUR",
        0x02 => "SET_MIN (UAC1) / RANGE (UAC2)",
        0x03 => "SET_MAX",
        0x04 => "SET_RES",
        0x05 => "SET_MEM",
        0x81 => "GET_CUR",
        0x82 => "GET_MIN / RANGE",
        0x83 => "GET_MAX",
        0x84 => "GET_RES",
        0x85 => "GET_MEM",
        0xFF => "GET_STAT",
        _ => "AUDIO_UNKNOWN",
    }
}

/// USB Video class-specific request decoder
fn usb_video_request_name(b_request: u8) -> &'static str {
    match b_request {
        0x01 => "SET_CUR",
        0x02 => "SET_CUR_ALL",
        0x81 => "GET_CUR",
        0x82 => "GET_MIN",
        0x83 => "GET_MAX",
        0x84 => "GET_RES",
        0x85 => "GET_LEN",
        0x86 => "GET_INFO",
        0x87 => "GET_DEF",
        0x88 => "GET_CUR_ALL",
        0x89 => "GET_MIN_ALL",
        0x8A => "GET_MAX_ALL",
        0x8B => "GET_RES_ALL",
        0x8C => "GET_DEF_ALL",
        _ => "UVC_UNKNOWN",
    }
}

/// Detect which USB class a setup packet belongs to (heuristic)
#[derive(Clone, Copy, Debug, PartialEq)]
enum UsbClassHint {
    Standard,
    CdcAcm,
    Hid,
    MassStorage,
    Audio,
    Video,
    Vendor,
    Unknown,
}

impl UsbClassHint {
    fn label(self) -> &'static str {
        match self {
            Self::Standard => "Standard",
            Self::CdcAcm => "CDC ACM",
            Self::Hid => "HID",
            Self::MassStorage => "Mass Storage",
            Self::Audio => "Audio",
            Self::Video => "Video",
            Self::Vendor => "Vendor",
            Self::Unknown => "Unknown",
        }
    }

    fn color(self) -> Color32 {
        match self {
            Self::Standard => Color32::from_rgb(130, 200, 255),
            Self::CdcAcm => Color32::from_rgb(100, 220, 160),
            Self::Hid => Color32::from_rgb(255, 200, 100),
            Self::MassStorage => Color32::from_rgb(200, 160, 255),
            Self::Audio => Color32::from_rgb(255, 160, 180),
            Self::Video => Color32::from_rgb(180, 220, 120),
            Self::Vendor => Color32::from_rgb(220, 180, 140),
            Self::Unknown => Color32::from_rgb(180, 180, 180),
        }
    }
}

/// Heuristic class detection from setup packet
fn detect_usb_class(
    bm_request_type: u8,
    b_request: u8,
    w_value: u16,
    w_index: u16,
) -> UsbClassHint {
    let req_type = (bm_request_type >> 5) & 0x03;
    match req_type {
        0 => UsbClassHint::Standard,
        2 => UsbClassHint::Vendor,
        1 => {
            // Class-specific: heuristic by request code
            match b_request {
                0x20..=0x23 => UsbClassHint::CdcAcm,
                0x01 | 0x02 | 0x03 | 0x09 | 0x0A | 0x0B if (w_value >> 8) <= 3 => UsbClassHint::Hid,
                0xFE | 0xFF => UsbClassHint::MassStorage,
                0x81..=0x85 | 0x01..=0x05 if w_index < 0x100 => {
                    // Audio vs Video: audio usually targets interface 0-1,
                    // video typically 2+; simplistic heuristic
                    if w_index <= 1 {
                        UsbClassHint::Audio
                    } else {
                        UsbClassHint::Video
                    }
                }
                _ => UsbClassHint::Unknown,
            }
        }
        _ => UsbClassHint::Unknown,
    }
}

/// Context-aware USB request name based on class
fn usb_class_request_name(class: UsbClassHint, b_request: u8) -> &'static str {
    match class {
        UsbClassHint::Standard => usb_request_name(b_request),
        UsbClassHint::CdcAcm => usb_cdc_request_name(b_request),
        UsbClassHint::Hid => usb_hid_request_name(b_request),
        UsbClassHint::MassStorage => usb_msc_request_name(b_request),
        UsbClassHint::Audio => usb_audio_request_name(b_request),
        UsbClassHint::Video => usb_video_request_name(b_request),
        UsbClassHint::Vendor => "Vendor-Defined",
        UsbClassHint::Unknown => usb_request_name(b_request),
    }
}

/// Parse USB device descriptor (18 bytes)
fn dissect_usb_device_descriptor(bytes: &[u8]) -> Vec<FrameField> {
    let mut fields = Vec::new();
    if bytes.len() < 18 {
        return fields;
    }
    let usb_ver = u16::from_le_bytes([bytes[2], bytes[3]]);
    let vendor = u16::from_le_bytes([bytes[8], bytes[9]]);
    let product = u16::from_le_bytes([bytes[10], bytes[11]]);
    let dev_ver = u16::from_le_bytes([bytes[12], bytes[13]]);
    fields.push(FrameField {
        name: "bLength".into(),
        start: 0,
        len: 1,
        decoded: format!("{}", bytes[0]),
        color_idx: 0,
    });
    fields.push(FrameField {
        name: "bDescriptorType".into(),
        start: 1,
        len: 1,
        decoded: format!("{} [DEVICE]", bytes[1]),
        color_idx: 1,
    });
    fields.push(FrameField {
        name: "bcdUSB".into(),
        start: 2,
        len: 2,
        decoded: format!("{}.{:02}", usb_ver >> 8, usb_ver & 0xFF),
        color_idx: 2,
    });
    fields.push(FrameField {
        name: "bDeviceClass".into(),
        start: 4,
        len: 1,
        decoded: format!("0x{:02X} [{}]", bytes[4], usb_class_name(bytes[4])),
        color_idx: 3,
    });
    fields.push(FrameField {
        name: "bDeviceSubClass".into(),
        start: 5,
        len: 1,
        decoded: format!("0x{:02X}", bytes[5]),
        color_idx: 4,
    });
    fields.push(FrameField {
        name: "bDeviceProtocol".into(),
        start: 6,
        len: 1,
        decoded: format!("0x{:02X}", bytes[6]),
        color_idx: 5,
    });
    fields.push(FrameField {
        name: "bMaxPacketSize0".into(),
        start: 7,
        len: 1,
        decoded: format!("{} bytes", bytes[7]),
        color_idx: 6,
    });
    fields.push(FrameField {
        name: "idVendor".into(),
        start: 8,
        len: 2,
        decoded: format!("0x{:04X}", vendor),
        color_idx: 7,
    });
    fields.push(FrameField {
        name: "idProduct".into(),
        start: 10,
        len: 2,
        decoded: format!("0x{:04X}", product),
        color_idx: 0,
    });
    fields.push(FrameField {
        name: "bcdDevice".into(),
        start: 12,
        len: 2,
        decoded: format!("{}.{:02}", dev_ver >> 8, dev_ver & 0xFF),
        color_idx: 1,
    });
    fields.push(FrameField {
        name: "iManufacturer".into(),
        start: 14,
        len: 1,
        decoded: format!("{}", bytes[14]),
        color_idx: 2,
    });
    fields.push(FrameField {
        name: "iProduct".into(),
        start: 15,
        len: 1,
        decoded: format!("{}", bytes[15]),
        color_idx: 3,
    });
    fields.push(FrameField {
        name: "iSerialNumber".into(),
        start: 16,
        len: 1,
        decoded: format!("{}", bytes[16]),
        color_idx: 4,
    });
    fields.push(FrameField {
        name: "bNumConfigurations".into(),
        start: 17,
        len: 1,
        decoded: format!("{}", bytes[17]),
        color_idx: 5,
    });
    fields
}

/// Parse USB configuration descriptor (9-byte header)
fn dissect_usb_config_descriptor(bytes: &[u8]) -> Vec<FrameField> {
    let mut fields = Vec::new();
    if bytes.len() < 9 {
        return fields;
    }
    let total_len = u16::from_le_bytes([bytes[2], bytes[3]]);
    let config_val = bytes[5];
    let attrs = bytes[7];
    let max_power_ma = bytes[8] as u16 * 2;
    fields.push(FrameField {
        name: "bLength".into(),
        start: 0,
        len: 1,
        decoded: format!("{}", bytes[0]),
        color_idx: 0,
    });
    fields.push(FrameField {
        name: "bDescriptorType".into(),
        start: 1,
        len: 1,
        decoded: "2 [CONFIGURATION]".into(),
        color_idx: 1,
    });
    fields.push(FrameField {
        name: "wTotalLength".into(),
        start: 2,
        len: 2,
        decoded: format!("{} bytes", total_len),
        color_idx: 2,
    });
    fields.push(FrameField {
        name: "bNumInterfaces".into(),
        start: 4,
        len: 1,
        decoded: format!("{}", bytes[4]),
        color_idx: 3,
    });
    fields.push(FrameField {
        name: "bConfigurationValue".into(),
        start: 5,
        len: 1,
        decoded: format!("{}", config_val),
        color_idx: 4,
    });
    fields.push(FrameField {
        name: "iConfiguration".into(),
        start: 6,
        len: 1,
        decoded: format!("{}", bytes[6]),
        color_idx: 5,
    });
    let self_powered = (attrs & 0x40) != 0;
    let remote_wakeup = (attrs & 0x20) != 0;
    fields.push(FrameField {
        name: "bmAttributes".into(),
        start: 7,
        len: 1,
        decoded: format!(
            "0x{:02X} [{}{}]",
            attrs,
            if self_powered {
                "SelfPowered "
            } else {
                "BusPowered "
            },
            if remote_wakeup { "RemoteWakeup" } else { "" }
        ),
        color_idx: 6,
    });
    fields.push(FrameField {
        name: "bMaxPower".into(),
        start: 8,
        len: 1,
        decoded: format!("{} mA", max_power_ma),
        color_idx: 7,
    });
    // Parse sub-descriptors if present
    let mut offset = 9;
    while offset + 2 <= bytes.len() {
        let sub_len = bytes[offset] as usize;
        let sub_type = bytes[offset + 1];
        if sub_len < 2 || offset + sub_len > bytes.len() {
            break;
        }
        match sub_type {
            4 if sub_len >= 9 => {
                // Interface descriptor
                fields.push(FrameField {
                    name: "Interface".into(),
                    start: offset,
                    len: sub_len,
                    decoded: format!(
                        "IF#{} Alt{} EPs={} Class=0x{:02X}[{}]",
                        bytes[offset + 2],
                        bytes[offset + 3],
                        bytes[offset + 4],
                        bytes[offset + 5],
                        usb_class_name(bytes[offset + 5])
                    ),
                    color_idx: 0,
                });
            }
            5 if sub_len >= 7 => {
                // Endpoint descriptor
                let ep_addr = bytes[offset + 2];
                let ep_attrs = bytes[offset + 3];
                let max_pkt = u16::from_le_bytes([bytes[offset + 4], bytes[offset + 5]]);
                let ep_dir = if (ep_addr & 0x80) != 0 { "IN" } else { "OUT" };
                fields.push(FrameField {
                    name: "Endpoint".into(),
                    start: offset,
                    len: sub_len,
                    decoded: format!(
                        "EP{} {} {} MaxPkt={}",
                        ep_addr & 0x0F,
                        ep_dir,
                        usb_transfer_type_name(ep_attrs),
                        max_pkt
                    ),
                    color_idx: 1,
                });
            }
            0x21 if sub_len >= 6 => {
                // HID descriptor
                fields.push(FrameField {
                    name: "HID Descriptor".into(),
                    start: offset,
                    len: sub_len,
                    decoded: format!(
                        "HID v{}.{:02} Country={} NumDesc={}",
                        bytes[offset + 2] >> 4,
                        bytes[offset + 2] & 0x0F,
                        bytes[offset + 4],
                        bytes[offset + 5]
                    ),
                    color_idx: 2,
                });
            }
            0x24 => {
                // CS_INTERFACE (CDC / Audio functional)
                let subtype = if sub_len >= 3 {
                    bytes[offset + 2]
                } else {
                    0xFF
                };
                fields.push(FrameField {
                    name: "CS_INTERFACE".into(),
                    start: offset,
                    len: sub_len,
                    decoded: format!(
                        "Subtype=0x{:02X} ({})",
                        subtype,
                        usb_cs_interface_name(subtype)
                    ),
                    color_idx: 3,
                });
            }
            _ => {
                fields.push(FrameField {
                    name: usb_descriptor_type_name(sub_type).to_string(),
                    start: offset,
                    len: sub_len,
                    decoded: format!("Type=0x{:02X} Len={}", sub_type, sub_len),
                    color_idx: 4,
                });
            }
        }
        offset += sub_len;
    }
    fields
}

/// CS_INTERFACE subtype names (CDC ACM)
fn usb_cs_interface_name(subtype: u8) -> &'static str {
    match subtype {
        0x00 => "Header",
        0x01 => "Call Management",
        0x02 => "ACM",
        0x03 => "Direct Line",
        0x04 => "Telephone Ringer",
        0x05 => "Telephone Call",
        0x06 => "Union",
        0x07 => "Country Selection",
        0x08 => "Telephone Operational",
        0x09 => "USB Terminal",
        0x0A => "Network Channel",
        0x0B => "Protocol Unit",
        0x0C => "Extension Unit",
        0x0D => "MCM",
        0x0E => "CAPI Control",
        0x0F => "Ethernet Networking",
        0x10 => "ATM Networking",
        0x12 => "MBIM",
        0x13 => "MBIM Extended",
        _ => "Unknown",
    }
}

/// Parse CDC SET_LINE_CODING data payload (7 bytes)
fn dissect_usb_cdc_line_coding(bytes: &[u8], offset: usize) -> Vec<FrameField> {
    let mut fields = Vec::new();
    if bytes.len() < offset + 7 {
        return fields;
    }
    let data = &bytes[offset..];
    let baud = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    let stop_bits = match data[4] {
        0 => "1",
        1 => "1.5",
        2 => "2",
        _ => "?",
    };
    let parity = match data[5] {
        0 => "None",
        1 => "Odd",
        2 => "Even",
        3 => "Mark",
        4 => "Space",
        _ => "?",
    };
    let data_bits = data[6];
    fields.push(FrameField {
        name: "dwDTERate".into(),
        start: offset,
        len: 4,
        decoded: format!("{} baud", baud),
        color_idx: 5,
    });
    fields.push(FrameField {
        name: "bCharFormat".into(),
        start: offset + 4,
        len: 1,
        decoded: format!("{} stop bits", stop_bits),
        color_idx: 6,
    });
    fields.push(FrameField {
        name: "bParityType".into(),
        start: offset + 5,
        len: 1,
        decoded: parity.to_string(),
        color_idx: 7,
    });
    fields.push(FrameField {
        name: "bDataBits".into(),
        start: offset + 6,
        len: 1,
        decoded: format!("{} bits", data_bits),
        color_idx: 0,
    });
    fields
}

/// Parse USB MSC BOT Command Block Wrapper (31 bytes signature 0x55534243)
fn dissect_usb_msc_cbw(bytes: &[u8]) -> Vec<FrameField> {
    let mut fields = Vec::new();
    if bytes.len() < 31 {
        return fields;
    }
    let sig = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let tag = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    let xfer_len = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
    let flags = bytes[12];
    let lun = bytes[13] & 0x0F;
    let cb_len = bytes[14] & 0x1F;
    let scsi_op = bytes[15];
    fields.push(FrameField {
        name: "dCBWSignature".into(),
        start: 0,
        len: 4,
        decoded: format!(
            "0x{:08X} {}",
            sig,
            if sig == 0x55534243 {
                "[USBC ✓]"
            } else {
                "[BAD]"
            }
        ),
        color_idx: 0,
    });
    fields.push(FrameField {
        name: "dCBWTag".into(),
        start: 4,
        len: 4,
        decoded: format!("0x{:08X}", tag),
        color_idx: 1,
    });
    fields.push(FrameField {
        name: "dCBWDataTransferLength".into(),
        start: 8,
        len: 4,
        decoded: format!("{} bytes", xfer_len),
        color_idx: 2,
    });
    fields.push(FrameField {
        name: "bmCBWFlags".into(),
        start: 12,
        len: 1,
        decoded: format!(
            "0x{:02X} [{}]",
            flags,
            if flags & 0x80 != 0 {
                "Data-IN"
            } else {
                "Data-OUT"
            }
        ),
        color_idx: 3,
    });
    fields.push(FrameField {
        name: "bCBWLUN".into(),
        start: 13,
        len: 1,
        decoded: format!("{}", lun),
        color_idx: 4,
    });
    fields.push(FrameField {
        name: "bCBWCBLength".into(),
        start: 14,
        len: 1,
        decoded: format!("{}", cb_len),
        color_idx: 5,
    });
    fields.push(FrameField {
        name: "SCSI OpCode".into(),
        start: 15,
        len: 1,
        decoded: format!("0x{:02X} [{}]", scsi_op, scsi_opcode_name(scsi_op)),
        color_idx: 6,
    });
    if cb_len > 1 {
        fields.push(FrameField {
            name: "CBWCB".into(),
            start: 16,
            len: (cb_len as usize).min(16),
            decoded: format!("{} bytes SCSI CDB", cb_len),
            color_idx: 7,
        });
    }
    fields
}

/// Parse USB MSC BOT Command Status Wrapper (13 bytes signature 0x55534253)
fn dissect_usb_msc_csw(bytes: &[u8]) -> Vec<FrameField> {
    let mut fields = Vec::new();
    if bytes.len() < 13 {
        return fields;
    }
    let sig = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let tag = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    let residue = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
    let status = bytes[12];
    fields.push(FrameField {
        name: "dCSWSignature".into(),
        start: 0,
        len: 4,
        decoded: format!(
            "0x{:08X} {}",
            sig,
            if sig == 0x55534253 {
                "[USBS ✓]"
            } else {
                "[BAD]"
            }
        ),
        color_idx: 0,
    });
    fields.push(FrameField {
        name: "dCSWTag".into(),
        start: 4,
        len: 4,
        decoded: format!("0x{:08X}", tag),
        color_idx: 1,
    });
    fields.push(FrameField {
        name: "dCSWDataResidue".into(),
        start: 8,
        len: 4,
        decoded: format!("{} bytes", residue),
        color_idx: 2,
    });
    let status_name = match status {
        0 => "Passed",
        1 => "Failed",
        2 => "Phase Error",
        _ => "Reserved",
    };
    fields.push(FrameField {
        name: "bCSWStatus".into(),
        start: 12,
        len: 1,
        decoded: format!("{} [{}]", status, status_name),
        color_idx: 3,
    });
    fields
}

/// Common SCSI opcode names
fn scsi_opcode_name(op: u8) -> &'static str {
    match op {
        0x00 => "TEST_UNIT_READY",
        0x03 => "REQUEST_SENSE",
        0x12 => "INQUIRY",
        0x1A => "MODE_SENSE(6)",
        0x1B => "START_STOP_UNIT",
        0x1E => "PREVENT_ALLOW_MEDIUM_REMOVAL",
        0x23 => "READ_FORMAT_CAPACITIES",
        0x25 => "READ_CAPACITY(10)",
        0x28 => "READ(10)",
        0x2A => "WRITE(10)",
        0x2F => "VERIFY(10)",
        0x35 => "SYNCHRONIZE_CACHE(10)",
        0x5A => "MODE_SENSE(10)",
        0xA8 => "READ(12)",
        0xAA => "WRITE(12)",
        _ => "UNKNOWN_SCSI",
    }
}

/// Detect USB BOT (Bulk-Only Transport) frames automatically
fn detect_usb_bot_frame(bytes: &[u8]) -> Option<&'static str> {
    if bytes.len() >= 31 {
        let sig = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        if sig == 0x55534243 {
            return Some("CBW");
        }
    }
    if bytes.len() >= 13 {
        let sig = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        if sig == 0x55534253 {
            return Some("CSW");
        }
    }
    None
}

/// Detect if data is a USB descriptor response (heuristic)
fn detect_usb_descriptor_type(bytes: &[u8]) -> Option<u8> {
    if bytes.len() >= 2 {
        let b_len = bytes[0] as usize;
        let b_type = bytes[1];
        if b_len >= 2 && b_len <= bytes.len() && matches!(b_type, 1..=11 | 0x21..=0x30) {
            return Some(b_type);
        }
    }
    None
}

// ══════════════════════════════════════════════════════════════�?// 帧深度解剖器 / Frame Dissector
// ══════════════════════════════════════════════════════════════�?
const FIELD_COLORS: [Color32; 8] = [
    Color32::from_rgb(100, 200, 255), // �?    Color32::from_rgb(120, 230, 150), // �?    Color32::from_rgb(255, 190, 100), // �?    Color32::from_rgb(200, 150, 255), // �?    Color32::from_rgb(255, 150, 150), // �?    Color32::from_rgb(150, 220, 220), // �?    Color32::from_rgb(250, 220, 130), // �?    Color32::from_rgb(220, 180, 200), // �?];

/// 协议帧字段描�?struct FrameField {
    name: String,
    start: usize,
    len: usize,
    decoded: String,
    color_idx: usize,
}

fn draw_frame_dissector(ui: &mut Ui, protocol: AnalyzerProtocol, bytes: &[u8]) {
    if bytes.is_empty() {
        return;
    }

    ui.label(
        RichText::new("帧结构深度解�?/ Frame Dissector")
            .strong()
            .size(13.0)
            .color(Color32::from_rgb(0, 122, 204)),
    );
    ui.add_space(4.0);

    let fields = dissect_protocol_fields(protocol, bytes);

    // 绘制着�?HEX 地图
    egui::Frame::group(ui.style())
        .fill(Color32::from_rgb(18, 22, 28))
        .stroke(egui::Stroke::new(1.0, Color32::from_rgb(50, 60, 70)))
        .corner_radius(6.0)
        .inner_margin(egui::Margin::symmetric(10, 8))
        .show(ui, |ui| {
            ui.label(
                RichText::new("Colored HEX Map")
                    .strong()
                    .color(Color32::from_rgb(160, 170, 185)),
            );

            // 行偏�?+ 着�?hex
            let bytes_per_row = 16usize;
            let mut row_start = 0usize;
            while row_start < bytes.len() {
                let row_end = (row_start + bytes_per_row).min(bytes.len());
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        RichText::new(format!("{:04X}�?, row_start))
                            .monospace()
                            .size(11.0)
                            .color(Color32::from_rgb(100, 110, 120)),
                    );
                    for (i, byte_val) in bytes.iter().enumerate().take(row_end).skip(row_start) {
                        let color = field_color_at(i, &fields);
                        ui.label(
                            RichText::new(format!("{:02X}", byte_val))
                                .monospace()
                                .size(11.5)
                                .color(color),
                        );
                    }
                    // ASCII �?                    ui.label(
                        RichText::new("�?)
                            .monospace()
                            .size(11.0)
                            .color(Color32::from_rgb(60, 66, 72)),
                    );
                    let ascii: String = bytes[row_start..row_end]
                        .iter()
                        .map(|&b| {
                            if b.is_ascii_graphic() || b == b' ' {
                                b as char
                            } else {
                                '.'
                            }
                        })
                        .collect();
                    ui.label(
                        RichText::new(ascii)
                            .monospace()
                            .size(11.0)
                            .color(Color32::from_rgb(140, 145, 155)),
                    );
                });
                row_start = row_end;
            }
        });

    ui.add_space(4.0);

    // 字段�?    if !fields.is_empty() {
        egui::Frame::group(ui.style())
            .fill(ui.visuals().faint_bg_color)
            .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
            .corner_radius(6.0)
            .inner_margin(egui::Margin::symmetric(10, 6))
            .show(ui, |ui| {
                ui.label(RichText::new("Field Breakdown").strong().size(12.0));
                ui.add_space(4.0);
                egui::Grid::new("frame_dissector_grid")
                    .num_columns(5)
                    .spacing([14.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label(RichText::new("�?).strong());
                        ui.label(RichText::new("Field").strong());
                        ui.label(RichText::new("Offset").strong());
                        ui.label(RichText::new("Raw HEX").strong());
                        ui.label(RichText::new("Decoded").strong());
                        ui.end_row();

                        for f in &fields {
                            let color = FIELD_COLORS[f.color_idx % FIELD_COLORS.len()];
                            ui.label(RichText::new("�?).color(color));
                            ui.label(RichText::new(&f.name).color(color));
                            ui.label(
                                RichText::new(format!("[{}..{}]", f.start, f.start + f.len))
                                    .monospace()
                                    .size(10.5)
                                    .color(Color32::from_rgb(130, 140, 150)),
                            );
                            let raw: Vec<String> = bytes
                                .get(f.start..f.start + f.len)
                                .unwrap_or(&[])
                                .iter()
                                .map(|b| format!("{:02X}", b))
                                .collect();
                            ui.label(
                                RichText::new(raw.join(" "))
                                    .monospace()
                                    .size(11.0)
                                    .color(Color32::from_rgb(200, 205, 210)),
                            );
                            ui.label(
                                RichText::new(&f.decoded)
                                    .size(11.0)
                                    .color(Color32::from_rgb(210, 220, 230)),
                            );
                            ui.end_row();
                        }
                    });
            });
    }

    // 位级视图（关键字段）
    draw_bitfield_view(ui, protocol, bytes);
}

fn field_color_at(byte_idx: usize, fields: &[FrameField]) -> Color32 {
    for f in fields {
        if byte_idx >= f.start && byte_idx < f.start + f.len {
            return FIELD_COLORS[f.color_idx % FIELD_COLORS.len()];
        }
    }
    Color32::from_rgb(160, 165, 175)
}

fn dissect_protocol_fields(protocol: AnalyzerProtocol, bytes: &[u8]) -> Vec<FrameField> {
    match protocol {
        AnalyzerProtocol::Serial => dissect_serial(bytes),
        AnalyzerProtocol::Tcp => dissect_tcp(bytes),
        AnalyzerProtocol::Udp => dissect_udp(bytes),
        AnalyzerProtocol::Can => dissect_can(bytes),
        AnalyzerProtocol::CanFd => dissect_can_fd(bytes),
        AnalyzerProtocol::ModbusRtu => dissect_modbus_rtu(bytes),
        AnalyzerProtocol::ModbusTcp => dissect_modbus_tcp(bytes),
        AnalyzerProtocol::Usb => dissect_usb(bytes),
    }
}

fn dissect_serial(bytes: &[u8]) -> Vec<FrameField> {
    let mut fields = Vec::new();
    // 尝试检测常见的帧结构（�?header + payload + checksum�?    if bytes.len() >= 2 {
        // 检测是否有常见头部标识
        if bytes[0] == 0xAA || bytes[0] == 0x55 || bytes[0] == 0x7E || bytes[0] == 0xFE {
            fields.push(FrameField {
                name: "Header".into(),
                start: 0,
                len: 1,
                decoded: format!("0x{:02X}", bytes[0]),
                color_idx: 0,
            });
            if bytes.len() >= 3 {
                let payload_end = bytes.len().saturating_sub(1);
                if payload_end > 1 {
                    fields.push(FrameField {
                        name: "Payload".into(),
                        start: 1,
                        len: payload_end - 1,
                        decoded: format!("{} bytes", payload_end - 1),
                        color_idx: 1,
                    });
                }
                fields.push(FrameField {
                    name: "Check/Tail".into(),
                    start: payload_end,
                    len: 1,
                    decoded: format!("0x{:02X}", bytes[payload_end]),
                    color_idx: 2,
                });
            }
        } else {
            // 通用: 区分可打印与不可打印�?            let mut seg_start = 0;
            let mut is_printable_seg = bytes[0].is_ascii_graphic() || bytes[0] == b' ';
            for i in 1..bytes.len() {
                let cur = bytes[i].is_ascii_graphic() || bytes[i] == b' ';
                if cur != is_printable_seg || i == bytes.len() - 1 {
                    let end = if i == bytes.len() - 1 && cur == is_printable_seg {
                        i + 1
                    } else {
                        i
                    };
                    let name = if is_printable_seg {
                        "ASCII Data"
                    } else {
                        "Binary Data"
                    };
                    fields.push(FrameField {
                        name: name.into(),
                        start: seg_start,
                        len: end - seg_start,
                        decoded: format!("{} bytes", end - seg_start),
                        color_idx: fields.len() % 6,
                    });
                    seg_start = i;
                    is_printable_seg = cur;
                }
            }
            if fields.is_empty() {
                fields.push(FrameField {
                    name: "Data".into(),
                    start: 0,
                    len: bytes.len(),
                    decoded: format!("{} bytes", bytes.len()),
                    color_idx: 0,
                });
            }
        }
    } else if bytes.len() == 1 {
        fields.push(FrameField {
            name: "Byte".into(),
            start: 0,
            len: 1,
            decoded: format!("0x{:02X}", bytes[0]),
            color_idx: 0,
        });
    }
    fields
}

fn dissect_tcp(bytes: &[u8]) -> Vec<FrameField> {
    // TCP 应用层帧（没有TCP头，那是操作系统处理的）
    let mut fields = Vec::new();
    if bytes.len() >= 4 {
        // 尝试检�?HTTP
        if bytes.starts_with(b"GET ") || bytes.starts_with(b"POST") || bytes.starts_with(b"HTTP") {
            let line_end = bytes
                .iter()
                .position(|&b| b == b'\r' || b == b'\n')
                .unwrap_or(bytes.len().min(80));
            fields.push(FrameField {
                name: "Request Line".into(),
                start: 0,
                len: line_end,
                decoded: String::from_utf8_lossy(&bytes[..line_end]).to_string(),
                color_idx: 0,
            });
            if line_end < bytes.len() {
                fields.push(FrameField {
                    name: "Body/Headers".into(),
                    start: line_end,
                    len: bytes.len() - line_end,
                    decoded: format!("{} bytes", bytes.len() - line_end),
                    color_idx: 1,
                });
            }
        } else {
            // 通用 TCP 数据
            let header_size = bytes.len().min(4);
            fields.push(FrameField {
                name: "TCP Header Hint".into(),
                start: 0,
                len: header_size,
                decoded: format!("{} B prefix", header_size),
                color_idx: 0,
            });
            if bytes.len() > header_size {
                fields.push(FrameField {
                    name: "TCP Payload".into(),
                    start: header_size,
                    len: bytes.len() - header_size,
                    decoded: format!("{} bytes", bytes.len() - header_size),
                    color_idx: 1,
                });
            }
        }
    } else {
        fields.push(FrameField {
            name: "Data".into(),
            start: 0,
            len: bytes.len(),
            decoded: format!("{} bytes", bytes.len()),
            color_idx: 0,
        });
    }
    fields
}

fn dissect_udp(bytes: &[u8]) -> Vec<FrameField> {
    let mut fields = Vec::new();
    if bytes.len() >= 2 {
        // 检测序列号（第一字节�?        fields.push(FrameField {
            name: "Sequence Hint".into(),
            start: 0,
            len: 1,
            decoded: format!("Seq={}", bytes[0]),
            color_idx: 0,
        });
        if bytes.len() > 1 {
            fields.push(FrameField {
                name: "UDP Payload".into(),
                start: 1,
                len: bytes.len() - 1,
                decoded: format!("{} bytes", bytes.len() - 1),
                color_idx: 1,
            });
        }
    } else {
        fields.push(FrameField {
            name: "Data".into(),
            start: 0,
            len: bytes.len(),
            decoded: format!("{} bytes", bytes.len()),
            color_idx: 0,
        });
    }
    fields
}

fn dissect_can(bytes: &[u8]) -> Vec<FrameField> {
    let mut fields = Vec::new();
    if bytes.len() >= 2 {
        // CAN 标准�? 2字节ID (11-bit) + data
        let can_id = (((bytes[0] as u16) << 3) | ((bytes[1] as u16) >> 5)) & 0x07FF;
        fields.push(FrameField {
            name: "CAN ID (11-bit)".into(),
            start: 0,
            len: 2,
            decoded: format!("0x{:03X} ({})", can_id, canopen_id_role(can_id)),
            color_idx: 0,
        });
        if bytes.len() > 2 {
            let dlc = bytes.len() - 2;
            fields.push(FrameField {
                name: "Data Field".into(),
                start: 2,
                len: dlc,
                decoded: format!("DLC={}", dlc),
                color_idx: 1,
            });
        }
    } else {
        // �?data �?        for (i, b) in bytes.iter().enumerate() {
            fields.push(FrameField {
                name: format!("D{}", i),
                start: i,
                len: 1,
                decoded: format!("0x{:02X}", b),
                color_idx: i % 4,
            });
        }
    }
    fields
}

fn dissect_can_fd(bytes: &[u8]) -> Vec<FrameField> {
    let mut fields = Vec::new();
    if bytes.len() >= 4 {
        let can_id = (((bytes[0] as u32) << 21)
            | ((bytes[1] as u32) << 13)
            | ((bytes[2] as u32) << 5)
            | ((bytes[3] as u32) >> 3))
            & 0x1FFF_FFFF;
        fields.push(FrameField {
            name: "CAN FD ID (29-bit)".into(),
            start: 0,
            len: 4,
            decoded: format!("0x{:08X}", can_id),
            color_idx: 0,
        });
        if bytes.len() > 4 {
            fields.push(FrameField {
                name: "FD Data".into(),
                start: 4,
                len: bytes.len() - 4,
                decoded: format!("DLC={} (FD)", bytes.len() - 4),
                color_idx: 1,
            });
        }
    } else {
        for (i, b) in bytes.iter().enumerate() {
            fields.push(FrameField {
                name: format!("D{}", i),
                start: i,
                len: 1,
                decoded: format!("0x{:02X}", b),
                color_idx: i % 4,
            });
        }
    }
    fields
}

fn dissect_modbus_rtu(bytes: &[u8]) -> Vec<FrameField> {
    let mut fields = Vec::new();
    if bytes.len() >= 4 {
        fields.push(FrameField {
            name: "Slave ID".into(),
            start: 0,
            len: 1,
            decoded: format!("Addr={}", bytes[0]),
            color_idx: 0,
        });
        let fc = bytes[1];
        let fc_name = modbus_fc_name(fc);
        fields.push(FrameField {
            name: "Function Code".into(),
            start: 1,
            len: 1,
            decoded: format!(
                "0x{:02X} {}{}",
                fc,
                fc_name,
                if fc & 0x80 != 0 { " [EXCEPTION]" } else { "" }
            ),
            color_idx: 1,
        });

        let crc_start = bytes.len().saturating_sub(2);
        if crc_start > 2 {
            fields.push(FrameField {
                name: "PDU Data".into(),
                start: 2,
                len: crc_start - 2,
                decoded: format!("{} bytes", crc_start - 2),
                color_idx: 2,
            });
        }

        // Modbus RTU 详细 PDU 解析
        if (fc & 0x80 == 0) && bytes.len() >= 6 {
            match fc {
                0x01..=0x04 => {
                    let start_addr = u16::from_be_bytes([bytes[2], bytes[3]]);
                    let qty = u16::from_be_bytes([bytes[4], bytes[5]]);
                    fields.push(FrameField {
                        name: "Start Address".into(),
                        start: 2,
                        len: 2,
                        decoded: format!("{} (0x{:04X})", start_addr, start_addr),
                        color_idx: 3,
                    });
                    fields.push(FrameField {
                        name: "Quantity".into(),
                        start: 4,
                        len: 2,
                        decoded: format!("{}", qty),
                        color_idx: 4,
                    });
                }
                0x05 | 0x06 => {
                    let addr = u16::from_be_bytes([bytes[2], bytes[3]]);
                    let val = u16::from_be_bytes([bytes[4], bytes[5]]);
                    fields.push(FrameField {
                        name: "Register Addr".into(),
                        start: 2,
                        len: 2,
                        decoded: format!("{} (0x{:04X})", addr, addr),
                        color_idx: 3,
                    });
                    fields.push(FrameField {
                        name: "Value".into(),
                        start: 4,
                        len: 2,
                        decoded: format!("{} (0x{:04X})", val, val),
                        color_idx: 4,
                    });
                }
                0x10 if bytes.len() >= 7 => {
                    let addr = u16::from_be_bytes([bytes[2], bytes[3]]);
                    let qty = u16::from_be_bytes([bytes[4], bytes[5]]);
                    let bc = bytes[6];
                    fields.push(FrameField {
                        name: "Start Address".into(),
                        start: 2,
                        len: 2,
                        decoded: format!("{}", addr),
                        color_idx: 3,
                    });
                    fields.push(FrameField {
                        name: "Reg Count".into(),
                        start: 4,
                        len: 2,
                        decoded: format!("{}", qty),
                        color_idx: 4,
                    });
                    fields.push(FrameField {
                        name: "Byte Count".into(),
                        start: 6,
                        len: 1,
                        decoded: format!("{}", bc),
                        color_idx: 5,
                    });
                    if bytes.len() > 7 && crc_start > 7 {
                        fields.push(FrameField {
                            name: "Register Values".into(),
                            start: 7,
                            len: crc_start - 7,
                            decoded: format!("{} bytes", crc_start - 7),
                            color_idx: 6,
                        });
                    }
                }
                _ => {}
            }
        }

        if crc_start + 2 <= bytes.len() {
            let crc_given = u16::from_le_bytes([bytes[crc_start], bytes[crc_start + 1]]);
            let crc_calc = crc16_modbus(&bytes[..crc_start]);
            let ok = crc_given == crc_calc;
            let crc_text = if ok {
                format!("0x{:04X} �?, crc_given)
            } else {
                format!("0x{:04X} �?expect 0x{:04X}", crc_given, crc_calc)
            };
            fields.push(FrameField {
                name: "CRC16".into(),
                start: crc_start,
                len: 2,
                decoded: crc_text,
                color_idx: 7,
            });
        }
    }
    fields
}

fn dissect_modbus_tcp(bytes: &[u8]) -> Vec<FrameField> {
    let mut fields = Vec::new();
    if bytes.len() >= 8 {
        let tx_id = u16::from_be_bytes([bytes[0], bytes[1]]);
        let proto = u16::from_be_bytes([bytes[2], bytes[3]]);
        let pdu_len = u16::from_be_bytes([bytes[4], bytes[5]]);
        let unit = bytes[6];
        let func = bytes[7];

        fields.push(FrameField {
            name: "Transaction ID".into(),
            start: 0,
            len: 2,
            decoded: format!("{}", tx_id),
            color_idx: 0,
        });
        fields.push(FrameField {
            name: "Protocol ID".into(),
            start: 2,
            len: 2,
            decoded: format!("{}{}", proto, if proto == 0 { " (Modbus)" } else { " �? }),
            color_idx: 1,
        });
        fields.push(FrameField {
            name: "Length".into(),
            start: 4,
            len: 2,
            decoded: format!("{} bytes", pdu_len),
            color_idx: 2,
        });
        fields.push(FrameField {
            name: "Unit ID".into(),
            start: 6,
            len: 1,
            decoded: format!("{}", unit),
            color_idx: 3,
        });
        fields.push(FrameField {
            name: "Function Code".into(),
            start: 7,
            len: 1,
            decoded: format!(
                "0x{:02X} {}{}",
                func,
                modbus_fc_name(func),
                if func & 0x80 != 0 { " [EXCEPTION]" } else { "" }
            ),
            color_idx: 4,
        });

        if bytes.len() > 8 {
            // 详细 PDU 解析
            match func & 0x7F {
                0x01..=0x04 if bytes.len() >= 12 => {
                    let addr = u16::from_be_bytes([bytes[8], bytes[9]]);
                    let qty = u16::from_be_bytes([bytes[10], bytes[11]]);
                    fields.push(FrameField {
                        name: "Start Address".into(),
                        start: 8,
                        len: 2,
                        decoded: format!("{} (0x{:04X})", addr, addr),
                        color_idx: 5,
                    });
                    fields.push(FrameField {
                        name: "Quantity".into(),
                        start: 10,
                        len: 2,
                        decoded: format!("{}", qty),
                        color_idx: 6,
                    });
                }
                _ => {
                    fields.push(FrameField {
                        name: "PDU Data".into(),
                        start: 8,
                        len: bytes.len() - 8,
                        decoded: format!("{} bytes", bytes.len() - 8),
                        color_idx: 5,
                    });
                }
            }
        }
    }
    fields
}

fn dissect_usb(bytes: &[u8]) -> Vec<FrameField> {
    let mut fields = Vec::new();

    // 1) 检�?MSC BOT CBW/CSW (不以 setup packet 开�?
    if let Some(bot_type) = detect_usb_bot_frame(bytes) {
        match bot_type {
            "CBW" => return dissect_usb_msc_cbw(bytes),
            "CSW" => return dissect_usb_msc_csw(bytes),
            _ => {}
        }
    }

    // 2) 检�?USB 描述符响�?    if let Some(desc_type) = detect_usb_descriptor_type(bytes) {
        match desc_type {
            1 => return dissect_usb_device_descriptor(bytes),
            2 => return dissect_usb_config_descriptor(bytes),
            _ => {
                fields.push(FrameField {
                    name: "Descriptor".into(),
                    start: 0,
                    len: bytes.len().min(bytes[0] as usize),
                    decoded: format!("{} Len={}", usb_descriptor_type_name(desc_type), bytes[0]),
                    color_idx: 0,
                });
                return fields;
            }
        }
    }

    // 3) Setup packet 解析 (8+ bytes)
    if bytes.len() >= 8 {
        let bm = bytes[0];
        let req = bytes[1];
        let w_value = u16::from_le_bytes([bytes[2], bytes[3]]);
        let w_index = u16::from_le_bytes([bytes[4], bytes[5]]);
        let w_len = u16::from_le_bytes([bytes[6], bytes[7]]);
        let (dir, typ, recipient) = decode_usb_bm_request_type(bm);
        let class_hint = detect_usb_class(bm, req, w_value, w_index);
        let class_req_name = usb_class_request_name(class_hint, req);

        fields.push(FrameField {
            name: "bmRequestType".into(),
            start: 0,
            len: 1,
            decoded: format!(
                "0x{:02X} {} {} {} [{}]",
                bm,
                dir,
                typ,
                recipient,
                class_hint.label()
            ),
            color_idx: 0,
        });
        fields.push(FrameField {
            name: "bRequest".into(),
            start: 1,
            len: 1,
            decoded: format!("0x{:02X} [{}]", req, class_req_name),
            color_idx: 1,
        });

        // Class-specific wValue decoding
        let w_value_decoded = match class_hint {
            UsbClassHint::Standard if req == 0x06 => {
                // GET_DESCRIPTOR: wValue = (DescType << 8) | DescIndex
                let desc_type = (w_value >> 8) as u8;
                let desc_idx = (w_value & 0xFF) as u8;
                format!(
                    "Type={} [{}] Index={}",
                    desc_type,
                    usb_descriptor_type_name(desc_type),
                    desc_idx
                )
            }
            UsbClassHint::Hid if req == 0x01 || req == 0x09 => {
                // GET_REPORT / SET_REPORT: wValue = (ReportType << 8) | ReportID
                let report_type = match (w_value >> 8) as u8 {
                    1 => "Input",
                    2 => "Output",
                    3 => "Feature",
                    _ => "?",
                };
                let report_id = (w_value & 0xFF) as u8;
                format!("ReportType={} ID={}", report_type, report_id)
            }
            _ => format!("0x{:04X} ({})", w_value, w_value),
        };
        fields.push(FrameField {
            name: "wValue".into(),
            start: 2,
            len: 2,
            decoded: w_value_decoded,
            color_idx: 2,
        });

        // Class-specific wIndex decoding
        let w_index_decoded = match class_hint {
            UsbClassHint::CdcAcm
            | UsbClassHint::Hid
            | UsbClassHint::Audio
            | UsbClassHint::Video => {
                format!("Interface={}", w_index & 0xFF)
            }
            UsbClassHint::Standard if recipient == "Endpoint" => {
                let ep_num = w_index & 0x0F;
                let ep_dir = if w_index & 0x80 != 0 { "IN" } else { "OUT" };
                format!("EP{} {}", ep_num, ep_dir)
            }
            _ => format!("0x{:04X} ({})", w_index, w_index),
        };
        fields.push(FrameField {
            name: "wIndex".into(),
            start: 4,
            len: 2,
            decoded: w_index_decoded,
            color_idx: 3,
        });
        fields.push(FrameField {
            name: "wLength".into(),
            start: 6,
            len: 2,
            decoded: format!("{} bytes", w_len),
            color_idx: 4,
        });

        // 4) Data Stage class-specific parsing
        if bytes.len() > 8 {
            let data_offset = 8;
            let data_len = bytes.len() - 8;
            match class_hint {
                UsbClassHint::CdcAcm if req == 0x20 || req == 0x21 => {
                    // SET_LINE_CODING / GET_LINE_CODING (7 bytes)
                    let mut cdc_fields = dissect_usb_cdc_line_coding(bytes, data_offset);
                    if cdc_fields.is_empty() {
                        fields.push(FrameField {
                            name: "CDC Data".into(),
                            start: data_offset,
                            len: data_len,
                            decoded: format!("{} bytes (line coding < 7B)", data_len),
                            color_idx: 5,
                        });
                    } else {
                        fields.append(&mut cdc_fields);
                    }
                }
                UsbClassHint::CdcAcm if req == 0x22 => {
                    // SET_CONTROL_LINE_STATE (wValue has DTR/RTS)
                    let dtr = (w_value & 0x01) != 0;
                    let rts = (w_value & 0x02) != 0;
                    fields.push(FrameField {
                        name: "LineState".into(),
                        start: 2,
                        len: 2,
                        decoded: format!(
                            "DTR={} RTS={}",
                            if dtr { "ON" } else { "OFF" },
                            if rts { "ON" } else { "OFF" }
                        ),
                        color_idx: 5,
                    });
                }
                _ => {
                    // Check if data stage contains a descriptor
                    if let Some(desc_type) = detect_usb_descriptor_type(&bytes[data_offset..]) {
                        fields.push(FrameField {
                            name: "Data (Descriptor)".into(),
                            start: data_offset,
                            len: data_len,
                            decoded: format!(
                                "{} [{}]",
                                usb_descriptor_type_name(desc_type),
                                data_len
                            ),
                            color_idx: 5,
                        });
                    } else {
                        fields.push(FrameField {
                            name: "Data Stage".into(),
                            start: data_offset,
                            len: data_len,
                            decoded: format!("{} bytes [{}]", data_len, class_hint.label()),
                            color_idx: 5,
                        });
                    }
                }
            }
        }
    }
    fields
}

/// 位级视图
fn draw_bitfield_view(ui: &mut Ui, protocol: AnalyzerProtocol, bytes: &[u8]) {
    if bytes.is_empty() {
        return;
    }

    ui.add_space(4.0);
    egui::Frame::group(ui.style())
        .fill(Color32::from_rgb(20, 24, 32))
        .stroke(egui::Stroke::new(1.0, Color32::from_rgb(45, 55, 65)))
        .corner_radius(6.0)
        .inner_margin(egui::Margin::symmetric(10, 6))
        .show(ui, |ui| {
            ui.label(
                RichText::new("Bit-Level View")
                    .strong()
                    .color(Color32::from_rgb(160, 170, 185)),
            );
            ui.add_space(2.0);

            // 显示�?4 字节的位级展开
            let show_bytes = bytes.len().min(4);
            for (idx, &b) in bytes.iter().enumerate().take(show_bytes) {
                let bits_str: String = (0..8)
                    .rev()
                    .map(|bit| if (b >> bit) & 1 == 1 { '1' } else { '0' })
                    .collect();

                let label = match protocol {
                    AnalyzerProtocol::ModbusRtu if idx == 0 => "Slave".to_string(),
                    AnalyzerProtocol::ModbusRtu if idx == 1 => "FC(err=b7)".to_string(),
                    AnalyzerProtocol::Usb if idx == 0 => "bmReqType".to_string(),
                    _ => format!("Byte[{}]", idx),
                };

                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        RichText::new(format!("{:>10} �?", label))
                            .monospace()
                            .size(10.5)
                            .color(Color32::from_rgb(130, 140, 155)),
                    );
                    for (bit_idx, ch) in bits_str.chars().enumerate() {
                        let bit_val = (b >> (7 - bit_idx)) & 1;
                        let color = if bit_val == 1 {
                            Color32::from_rgb(100, 220, 160)
                        } else {
                            Color32::from_rgb(90, 95, 105)
                        };
                        ui.label(
                            RichText::new(format!("{}", ch))
                                .monospace()
                                .size(12.0)
                                .color(color),
                        );
                    }
                    ui.label(
                        RichText::new(format!(" �?0x{:02X} ({})", b, b))
                            .monospace()
                            .size(10.5)
                            .color(Color32::from_rgb(140, 150, 165)),
                    );
                });
            }

            // 位位置标�?            ui.horizontal_wrapped(|ui| {
                ui.label(
                    RichText::new("           �?")
                        .monospace()
                        .size(10.5)
                        .color(Color32::from_rgb(70, 75, 85)),
                );
                for i in (0..8).rev() {
                    ui.label(
                        RichText::new(format!("{}", i))
                            .monospace()
                            .size(9.5)
                            .color(Color32::from_rgb(80, 85, 95)),
                    );
                }
                ui.label(
                    RichText::new(" �?MSB..LSB")
                        .monospace()
                        .size(9.5)
                        .color(Color32::from_rgb(80, 85, 95)),
                );
            });
        });
}

// ══════════════════════════════════════════════════════════════�?// 事务分析 / Transaction Analysis
// ══════════════════════════════════════════════════════════════�?
fn draw_transaction_analysis(
    ui: &mut Ui,
    protocol: AnalyzerProtocol,
    entries: &[crate::app::LogEntry],
) {
    if entries.len() < 2 {
        return;
    }

    ui.label(
        RichText::new("事务分析 / Transaction Tracker")
            .strong()
            .size(13.0)
            .color(Color32::from_rgb(0, 122, 204)),
    );
    ui.add_space(4.0);

    let transactions = match protocol {
        AnalyzerProtocol::ModbusRtu | AnalyzerProtocol::ModbusTcp => {
            detect_modbus_transactions(entries, protocol)
        }
        AnalyzerProtocol::Can | AnalyzerProtocol::CanFd => detect_can_transactions(entries),
        _ => detect_generic_transactions(entries),
    };

    if transactions.is_empty() {
        ui.label(
            RichText::new("暂未检测到请求-响应事务配对").color(Color32::from_rgb(160, 160, 175)),
        );
        return;
    }

    egui::Frame::group(ui.style())
        .fill(ui.visuals().faint_bg_color)
        .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
        .corner_radius(6.0)
        .inner_margin(egui::Margin::symmetric(10, 6))
        .show(ui, |ui| {
            let completed = transactions.iter().filter(|t| t.completed).count();
            let timed_out = transactions.len() - completed;
            let avg_rt: f32 = if completed > 0 {
                transactions
                    .iter()
                    .filter(|t| t.completed)
                    .map(|t| t.response_time_ms)
                    .sum::<f32>()
                    / completed as f32
            } else {
                0.0
            };

            ui.horizontal_wrapped(|ui| {
                draw_badge(ui, &format!("Total: {}", transactions.len()), true);
                draw_badge(ui, &format!("Complete: {}", completed), completed > 0);
                draw_badge(ui, &format!("Timeout: {}", timed_out), timed_out == 0);
                draw_badge(ui, &format!("Avg RT: {:.1}ms", avg_rt), avg_rt < 100.0);
            });
            ui.add_space(4.0);

            egui::ScrollArea::vertical()
                .max_height(140.0)
                .show(ui, |ui| {
                    egui::Grid::new("transaction_grid")
                        .num_columns(5)
                        .spacing([12.0, 3.0])
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label(RichText::new("#").strong());
                            ui.label(RichText::new("Request").strong());
                            ui.label(RichText::new("Response").strong());
                            ui.label(RichText::new("RT(ms)").strong());
                            ui.label(RichText::new("Status").strong());
                            ui.end_row();

                            for (i, t) in transactions.iter().take(50).enumerate() {
                                ui.label(
                                    RichText::new(format!("{}", i + 1)).monospace().size(10.5),
                                );
                                ui.label(
                                    RichText::new(&t.request_summary)
                                        .monospace()
                                        .size(10.5)
                                        .color(Color32::from_rgb(100, 200, 255)),
                                );
                                ui.label(
                                    RichText::new(&t.response_summary)
                                        .monospace()
                                        .size(10.5)
                                        .color(if t.completed {
                                            Color32::from_rgb(120, 230, 150)
                                        } else {
                                            Color32::from_rgb(255, 160, 100)
                                        }),
                                );
                                ui.label(
                                    RichText::new(if t.completed {
                                        format!("{:.1}", t.response_time_ms)
                                    } else {
                                        "�?.into()
                                    })
                                    .monospace()
                                    .size(10.5),
                                );
                                let (status_text, status_ok) = if t.completed {
                                    if t.error {
                                        ("ERR", false)
                                    } else {
                                        ("OK", true)
                                    }
                                } else {
                                    ("TIMEOUT", false)
                                };
                                draw_badge(ui, status_text, status_ok);
                                ui.end_row();
                            }
                        });
                });
        });
}

struct TransactionPair {
    request_summary: String,
    response_summary: String,
    response_time_ms: f32,
    completed: bool,
    error: bool,
}

fn detect_modbus_transactions(
    entries: &[crate::app::LogEntry],
    protocol: AnalyzerProtocol,
) -> Vec<TransactionPair> {
    let mut transactions = Vec::new();
    let tx_entries: Vec<_> = entries
        .iter()
        .filter(|e| matches!(e.direction, LogDirection::Tx))
        .collect();
    let rx_entries: Vec<_> = entries
        .iter()
        .filter(|e| matches!(e.direction, LogDirection::Rx))
        .collect();

    let min_frame = if matches!(protocol, AnalyzerProtocol::ModbusTcp) {
        8
    } else {
        4
    };

    for tx in tx_entries.iter().take(50) {
        if tx.data.len() < min_frame {
            continue;
        }

        let (slave, fc) = if matches!(protocol, AnalyzerProtocol::ModbusTcp) && tx.data.len() >= 8 {
            (tx.data[6], tx.data[7])
        } else {
            (tx.data[0], tx.data[1])
        };

        let req_summary = format!("S={} FC=0x{:02X} [{}]", slave, fc, modbus_fc_name(fc));

        // 寻找匹配的响�?        let mut found = false;
        for rx in &rx_entries {
            if rx.data.len() < min_frame {
                continue;
            }
            let (r_slave, r_fc) =
                if matches!(protocol, AnalyzerProtocol::ModbusTcp) && rx.data.len() >= 8 {
                    (rx.data[6], rx.data[7])
                } else {
                    (rx.data[0], rx.data[1])
                };

            if r_slave == slave && (r_fc == fc || r_fc == fc | 0x80) {
                let rt = calc_entry_delta_ms(tx, rx);
                let is_err = (r_fc & 0x80) != 0;
                let resp_summary = if is_err {
                    format!("S={} FC=0x{:02X} [Exception]", r_slave, r_fc)
                } else {
                    format!(
                        "S={} FC=0x{:02X} [OK, {} bytes]",
                        r_slave,
                        r_fc,
                        rx.data.len()
                    )
                };
                transactions.push(TransactionPair {
                    request_summary: req_summary.clone(),
                    response_summary: resp_summary,
                    response_time_ms: rt,
                    completed: true,
                    error: is_err,
                });
                found = true;
                break;
            }
        }

        if !found {
            transactions.push(TransactionPair {
                request_summary: req_summary,
                response_summary: "�?.into(),
                response_time_ms: 0.0,
                completed: false,
                error: false,
            });
        }
    }
    transactions
}

fn detect_can_transactions(entries: &[crate::app::LogEntry]) -> Vec<TransactionPair> {
    let mut transactions = Vec::new();
    let tx_entries: Vec<_> = entries
        .iter()
        .filter(|e| matches!(e.direction, LogDirection::Tx))
        .collect();
    let rx_entries: Vec<_> = entries
        .iter()
        .filter(|e| matches!(e.direction, LogDirection::Rx))
        .collect();

    for tx in tx_entries.iter().take(50) {
        if tx.data.is_empty() {
            continue;
        }
        let tx_hex: String = tx
            .data
            .iter()
            .take(4)
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ");
        let req_summary = format!("[{}B] {}", tx.data.len(), tx_hex);

        let mut found = false;
        for rx in &rx_entries {
            // 简单匹�? 相同长度的响�?            let rt = calc_entry_delta_ms(tx, rx);
            if rt > 0.0 && rt < 500.0 {
                let rx_hex: String = rx
                    .data
                    .iter()
                    .take(4)
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" ");
                transactions.push(TransactionPair {
                    request_summary: req_summary.clone(),
                    response_summary: format!("[{}B] {}", rx.data.len(), rx_hex),
                    response_time_ms: rt,
                    completed: true,
                    error: false,
                });
                found = true;
                break;
            }
        }
        if !found {
            transactions.push(TransactionPair {
                request_summary: req_summary,
                response_summary: "�?.into(),
                response_time_ms: 0.0,
                completed: false,
                error: false,
            });
        }
    }
    transactions
}

fn detect_generic_transactions(entries: &[crate::app::LogEntry]) -> Vec<TransactionPair> {
    let mut transactions = Vec::new();
    let mut i = 0;
    while i < entries.len() && transactions.len() < 50 {
        if matches!(entries[i].direction, LogDirection::Tx) {
            let tx_hex: String = entries[i]
                .data
                .iter()
                .take(6)
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" ");
            // 找下一�?RX
            let mut found = false;
            for j in (i + 1)..entries.len().min(i + 20) {
                if matches!(entries[j].direction, LogDirection::Rx) {
                    let rt = calc_entry_delta_ms(&entries[i], &entries[j]);
                    let rx_hex: String = entries[j]
                        .data
                        .iter()
                        .take(6)
                        .map(|b| format!("{:02X}", b))
                        .collect::<Vec<_>>()
                        .join(" ");
                    transactions.push(TransactionPair {
                        request_summary: format!("[{}B] {}", entries[i].data.len(), tx_hex),
                        response_summary: format!("[{}B] {}", entries[j].data.len(), rx_hex),
                        response_time_ms: rt,
                        completed: true,
                        error: false,
                    });
                    found = true;
                    i = j + 1;
                    break;
                }
            }
            if !found {
                transactions.push(TransactionPair {
                    request_summary: format!("[{}B] {}", entries[i].data.len(), tx_hex),
                    response_summary: "�?.into(),
                    response_time_ms: 0.0,
                    completed: false,
                    error: false,
                });
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    transactions
}

fn calc_entry_delta_ms(a: &crate::app::LogEntry, b: &crate::app::LogEntry) -> f32 {
    let ta = parse_ts_ms(&a.timestamp);
    let tb = parse_ts_ms(&b.timestamp);
    match (ta, tb) {
        (Some(a_ms), Some(b_ms)) => b_ms.saturating_sub(a_ms) as f32,
        _ => 0.0,
    }
}

// ══════════════════════════════════════════════════════════════�?// 工具函数
// ══════════════════════════════════════════════════════════════�?
fn export_filtered_csv(entries: &[crate::app::LogEntry]) -> anyhow::Result<std::path::PathBuf> {
    let ts = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let path = std::env::temp_dir().join(format!("protocol_analysis_{}.csv", ts));
    let mut out = String::from("timestamp,channel,direction,len,hex\n");
    for e in entries {
        let dir = match e.direction {
            LogDirection::Tx => "TX",
            LogDirection::Rx => "RX",
            LogDirection::Info => "INFO",
        };
        let hex = e
            .data
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ");
        out.push_str(&format!(
            "\"{}\",\"{}\",\"{}\",{},\"{}\"\n",
            e.timestamp,
            e.channel,
            dir,
            e.data.len(),
            hex
        ));
    }
    std::fs::write(&path, out)?;
    Ok(path)
}

fn xor8(data: &[u8]) -> u8 {
    data.iter().fold(0u8, |acc, b| acc ^ b)
}

fn sum8(data: &[u8]) -> u8 {
    data.iter().fold(0u8, |acc, b| acc.wrapping_add(*b))
}

fn crc16_modbus(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &byte in data {
        crc ^= byte as u16;
        for _ in 0..8 {
            if crc & 0x0001 != 0 {
                crc = (crc >> 1) ^ 0xA001;
            } else {
                crc >>= 1;
            }
        }
    }
    crc
}

fn entropy(data: &[u8]) -> f32 {
    if data.is_empty() {
        return 0.0;
    }
    let mut freq = [0usize; 256];
    for &b in data {
        freq[b as usize] += 1;
    }
    let len = data.len() as f32;
    let mut h = 0.0f32;
    for &f in &freq {
        if f == 0 {
            continue;
        }
        let p = f as f32 / len;
        h -= p * p.log2();
    }
    h
}

fn format_data_with_mode(data: &[u8], mode: DisplayMode) -> String {
    match mode {
        DisplayMode::Hex => data
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" "),
        DisplayMode::Ascii => String::from_utf8_lossy(data).to_string(),
        DisplayMode::Mixed => {
            let hex = data
                .iter()
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" ");
            let ascii: String = data
                .iter()
                .map(|&b| {
                    if b.is_ascii_graphic() || b == b' ' {
                        b as char
                    } else {
                        '.'
                    }
                })
                .collect();
            format!("{} | {}", hex, ascii)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::LogEntry;

    fn log(ts: &str, dir: LogDirection, bytes: &[u8], channel: &str) -> LogEntry {
        LogEntry {
            timestamp: ts.to_string(),
            direction: dir,
            data: bytes.to_vec(),
            display_mode: DisplayMode::Hex,
            channel: channel.to_string(),
        }
    }

    #[test]
    fn test_crc16_modbus_known_vector() {
        let payload = [0x01u8, 0x03, 0x00, 0x00, 0x00, 0x0A];
        assert_eq!(crc16_modbus(&payload), 0xCDC5);
    }

    #[test]
    fn test_inter_frame_intervals() {
        let entries = vec![
            log("12:00:00.000", LogDirection::Rx, &[1], "TCP"),
            log("12:00:00.010", LogDirection::Rx, &[2], "TCP"),
            log("12:00:00.025", LogDirection::Rx, &[3], "TCP"),
        ];
        let d = inter_frame_intervals_ms(&entries);
        assert_eq!(d.len(), 2);
        assert!((d[0] - 10.0).abs() < 0.1);
        assert!((d[1] - 15.0).abs() < 0.1);
    }

    #[test]
    fn test_duplicate_payload_ratio() {
        let entries = vec![
            log("12:00:00.000", LogDirection::Tx, &[0xAA], "TCP"),
            log("12:00:00.001", LogDirection::Tx, &[0xAA], "TCP"),
            log("12:00:00.002", LogDirection::Tx, &[0xBB], "TCP"),
        ];
        let r = duplicate_payload_ratio(&entries);
        assert!(r > 30.0 && r < 40.0);
    }

    #[test]
    fn test_protocol_payload_capacity() {
        assert_eq!(protocol_payload_capacity(AnalyzerProtocol::Can), 8);
        assert_eq!(protocol_payload_capacity(AnalyzerProtocol::CanFd), 64);
        assert_eq!(protocol_payload_capacity(AnalyzerProtocol::Tcp), 1460);
    }

    #[test]
    fn test_can_fd_len_map() {
        assert!(is_can_fd_len_valid(8));
        assert!(is_can_fd_len_valid(12));
        assert!(is_can_fd_len_valid(64));
        assert!(!is_can_fd_len_valid(9));
        assert!(!is_can_fd_len_valid(63));
    }

    #[test]
    fn test_usb_bm_request_decode() {
        let (dir, typ, recipient) = decode_usb_bm_request_type(0x80);
        assert_eq!(dir, "Device->Host");
        assert_eq!(typ, "Standard");
        assert_eq!(recipient, "Device");

        let (dir2, typ2, recipient2) = decode_usb_bm_request_type(0x21);
        assert_eq!(dir2, "Host->Device");
        assert_eq!(typ2, "Class");
        assert_eq!(recipient2, "Interface");
    }

    #[test]
    fn test_udp_sequence_disorder_detection() {
        let good = vec![
            log("12:00:00.000", LogDirection::Rx, &[1, 0xAA], "UDP"),
            log("12:00:00.001", LogDirection::Rx, &[2, 0xAA], "UDP"),
            log("12:00:00.002", LogDirection::Rx, &[3, 0xAA], "UDP"),
            log("12:00:00.003", LogDirection::Rx, &[4, 0xAA], "UDP"),
        ];
        assert!(!infer_sequence_disorder(&good));

        let bad = vec![
            log("12:00:00.000", LogDirection::Rx, &[1, 0xAA], "UDP"),
            log("12:00:00.001", LogDirection::Rx, &[7, 0xAA], "UDP"),
            log("12:00:00.002", LogDirection::Rx, &[3, 0xAA], "UDP"),
            log("12:00:00.003", LogDirection::Rx, &[9, 0xAA], "UDP"),
        ];
        assert!(infer_sequence_disorder(&bad));
    }

    #[test]
    fn test_modbus_tcp_diagnostics() {
        let kpi = IndustrialKpi {
            payload_utilization_pct: 20.0,
            frame_error_rate_pct: 0.0,
            bit_error_rate_ppm: 0.0,
            avg_inter_frame_ms: 2.0,
            jitter_ms: 1.0,
            duplicate_payload_ratio_pct: 0.0,
        };

        let ok = [
            0x00, 0x01, 0x00, 0x00, 0x00, 0x06, 0x11, 0x03, 0x00, 0x6B, 0x00, 0x03,
        ];
        let checks = modbus_tcp_checks(&ok, &kpi);
        assert!(checks.iter().any(|c| c.name == "MBAP header" && c.pass));
        assert!(checks.iter().any(|c| c.name == "Protocol ID" && c.pass));
        assert!(checks
            .iter()
            .any(|c| c.name == "Length consistency" && c.pass));

        let bad = [0x00, 0x01, 0x00, 0x02, 0x00, 0x06, 0x11, 0x83, 0x02];
        let bad_checks = modbus_tcp_checks(&bad, &kpi);
        assert!(bad_checks
            .iter()
            .any(|c| c.name == "Protocol ID" && !c.pass));
        assert!(bad_checks
            .iter()
            .any(|c| c.name == "Exception response" && !c.pass));
    }

    #[test]
    fn test_protocol_checks_matrix_non_empty() {
        let entries = vec![log(
            "12:00:00.000",
            LogDirection::Rx,
            &[0x01, 0x02, 0x03],
            "Serial",
        )];
        let kpi = IndustrialKpi {
            payload_utilization_pct: 10.0,
            frame_error_rate_pct: 0.1,
            bit_error_rate_ppm: 1.0,
            avg_inter_frame_ms: 3.0,
            jitter_ms: 1.0,
            duplicate_payload_ratio_pct: 0.0,
        };

        for protocol in AnalyzerProtocol::all() {
            let checks =
                protocol_diagnostic_checks(*protocol, &[0x01, 0x02, 0x03, 0x04], &entries, &kpi);
            assert!(!checks.is_empty());
        }
    }

    // ══════════════════════════════════════════════════════════�?    // USB 协议特化分析引擎测试
    // ══════════════════════════════════════════════════════════�?
    #[test]
    fn test_usb_class_name_mapping() {
        assert_eq!(usb_class_name(0x02), "CDC (Comm)");
        assert_eq!(usb_class_name(0x03), "HID");
        assert_eq!(usb_class_name(0x08), "Mass Storage");
        assert_eq!(usb_class_name(0x01), "Audio");
        assert_eq!(usb_class_name(0x0E), "Video");
        assert_eq!(usb_class_name(0xFF), "Vendor-Specific");
        assert_eq!(usb_class_name(0x09), "Hub");
    }

    #[test]
    fn test_usb_descriptor_type_names() {
        assert_eq!(usb_descriptor_type_name(1), "DEVICE");
        assert_eq!(usb_descriptor_type_name(2), "CONFIGURATION");
        assert_eq!(usb_descriptor_type_name(4), "INTERFACE");
        assert_eq!(usb_descriptor_type_name(5), "ENDPOINT");
        assert_eq!(usb_descriptor_type_name(0x21), "HID");
        assert_eq!(usb_descriptor_type_name(0x24), "CS_INTERFACE");
    }

    #[test]
    fn test_usb_pid_names() {
        assert_eq!(usb_pid_name(0x01), "OUT");
        assert_eq!(usb_pid_name(0x09), "IN");
        assert_eq!(usb_pid_name(0x0D), "SETUP");
        assert_eq!(usb_pid_name(0x03), "DATA0");
        assert_eq!(usb_pid_name(0x0B), "DATA1");
        assert_eq!(usb_pid_name(0x02), "ACK");
        assert_eq!(usb_pid_name(0x0A), "NAK");
        assert_eq!(usb_pid_name(0x0E), "STALL");
    }

    #[test]
    fn test_usb_cdc_request_names() {
        assert_eq!(usb_cdc_request_name(0x20), "SET_LINE_CODING");
        assert_eq!(usb_cdc_request_name(0x21), "GET_LINE_CODING");
        assert_eq!(usb_cdc_request_name(0x22), "SET_CONTROL_LINE_STATE");
        assert_eq!(usb_cdc_request_name(0x23), "SEND_BREAK");
    }

    #[test]
    fn test_usb_hid_request_names() {
        assert_eq!(usb_hid_request_name(0x01), "GET_REPORT");
        assert_eq!(usb_hid_request_name(0x09), "SET_REPORT");
        assert_eq!(usb_hid_request_name(0x0A), "SET_IDLE");
        assert_eq!(usb_hid_request_name(0x0B), "SET_PROTOCOL");
    }

    #[test]
    fn test_usb_msc_request_names() {
        assert_eq!(usb_msc_request_name(0xFE), "GET_MAX_LUN");
        assert_eq!(usb_msc_request_name(0xFF), "BULK_ONLY_RESET");
    }

    #[test]
    fn test_usb_audio_video_request_names() {
        assert_eq!(usb_audio_request_name(0x01), "SET_CUR");
        assert_eq!(usb_audio_request_name(0x81), "GET_CUR");
        assert_eq!(usb_video_request_name(0x81), "GET_CUR");
        assert_eq!(usb_video_request_name(0x87), "GET_DEF");
    }

    #[test]
    fn test_usb_class_detection_standard() {
        assert_eq!(
            detect_usb_class(0x80, 0x06, 0x0100, 0x0000),
            UsbClassHint::Standard
        );
        assert_eq!(
            detect_usb_class(0x00, 0x09, 0x0001, 0x0000),
            UsbClassHint::Standard
        );
    }

    #[test]
    fn test_usb_class_detection_cdc() {
        assert_eq!(
            detect_usb_class(0x21, 0x20, 0x0000, 0x0000),
            UsbClassHint::CdcAcm
        );
        assert_eq!(
            detect_usb_class(0x21, 0x22, 0x0003, 0x0001),
            UsbClassHint::CdcAcm
        );
    }

    #[test]
    fn test_usb_class_detection_hid() {
        assert_eq!(
            detect_usb_class(0xA1, 0x01, 0x0100, 0x0000),
            UsbClassHint::Hid
        );
        assert_eq!(
            detect_usb_class(0x21, 0x09, 0x0200, 0x0000),
            UsbClassHint::Hid
        );
    }

    #[test]
    fn test_usb_class_detection_vendor() {
        assert_eq!(
            detect_usb_class(0x40, 0x01, 0x0000, 0x0000),
            UsbClassHint::Vendor
        );
        assert_eq!(
            detect_usb_class(0xC0, 0x55, 0x1234, 0x5678),
            UsbClassHint::Vendor
        );
    }

    #[test]
    fn test_dissect_usb_setup_cdc_set_line_coding() {
        // SET_LINE_CODING: 115200-8N1, bmReqType=0x21, bReq=0x20
        let mut pkt = vec![0x21, 0x20, 0x00, 0x00, 0x00, 0x00, 0x07, 0x00];
        // data stage: 115200 baud, 0 stop, 0 parity, 8 databits
        let baud: u32 = 115200;
        pkt.extend_from_slice(&baud.to_le_bytes());
        pkt.push(0x00); // stop
        pkt.push(0x00); // parity
        pkt.push(0x08); // databits
        let fields = dissect_usb(&pkt);
        assert!(fields.len() >= 5, "CDC dissect should have >= 5 fields");
        assert!(
            fields[0].decoded.contains("CDC ACM"),
            "Should detect CDC ACM class"
        );
        assert!(
            fields[1].decoded.contains("SET_LINE_CODING"),
            "Should detect SET_LINE_CODING"
        );
        // data stage should have line coding fields
        assert!(
            fields
                .iter()
                .any(|f| f.name == "dwDTERate" && f.decoded.contains("115200")),
            "Should parse baud rate 115200"
        );
    }

    #[test]
    fn test_dissect_usb_setup_hid_get_report() {
        // HID GET_REPORT: bmReqType=0xA1, bReq=0x01, wValue=0x0100 (Input, ID=0)
        let pkt = [0xA1, 0x01, 0x00, 0x01, 0x00, 0x00, 0x08, 0x00];
        let fields = dissect_usb(&pkt);
        assert!(fields.len() >= 5);
        assert!(fields[0].decoded.contains("HID"));
        assert!(fields[1].decoded.contains("GET_REPORT"));
        assert!(
            fields[2].decoded.contains("Input"),
            "wValue should decode report type"
        );
    }

    #[test]
    fn test_dissect_usb_msc_cbw() {
        let mut cbw = vec![0u8; 31];
        // Signature: USBC
        cbw[0] = 0x43;
        cbw[1] = 0x42;
        cbw[2] = 0x53;
        cbw[3] = 0x55;
        cbw[4] = 0x01; // tag
        cbw[8] = 0x00;
        cbw[9] = 0x02; // xfer_len = 512
        cbw[12] = 0x80; // Data-IN
        cbw[13] = 0x00; // LUN 0
        cbw[14] = 0x0A; // CB length = 10
        cbw[15] = 0x28; // SCSI READ(10)
        let fields = dissect_usb(&cbw);
        assert!(!fields.is_empty(), "CBW should produce fields");
        assert!(
            fields[0].decoded.contains("USBC"),
            "Should detect CBW signature"
        );
        assert!(
            fields.iter().any(|f| f.decoded.contains("READ(10)")),
            "Should parse SCSI opcode"
        );
    }

    #[test]
    fn test_dissect_usb_msc_csw() {
        let mut csw = vec![0u8; 13];
        // Signature: USBS
        csw[0] = 0x53;
        csw[1] = 0x42;
        csw[2] = 0x53;
        csw[3] = 0x55;
        csw[4] = 0x01; // tag
        csw[12] = 0x00; // status = Passed
        let fields = dissect_usb(&csw);
        assert!(!fields.is_empty(), "CSW should produce fields");
        assert!(
            fields[0].decoded.contains("USBS"),
            "Should detect CSW signature"
        );
        assert!(
            fields.iter().any(|f| f.decoded.contains("Passed")),
            "Status should be Passed"
        );
    }

    #[test]
    fn test_dissect_usb_device_descriptor() {
        // Minimal 18-byte USB device descriptor
        let desc: [u8; 18] = [
            18, 1, // bLength=18, bDescriptorType=DEVICE
            0x00, 0x02, // bcdUSB = 2.00
            0x02, 0x00, 0x00, // class=CDC, subclass=0, protocol=0
            64,   // bMaxPacketSize0
            0x83, 0x04, // idVendor = 0x0483 (ST)
            0x40, 0x57, // idProduct = 0x5740
            0x00, 0x02, // bcdDevice = 2.00
            1, 2, 3, // iManufacturer, iProduct, iSerialNumber
            1, // bNumConfigurations
        ];
        let fields = dissect_usb(&desc);
        assert!(
            fields.len() >= 14,
            "Device descriptor should have 14 fields"
        );
        assert!(fields[0].decoded.contains("18"), "bLength = 18");
        assert!(fields[3].decoded.contains("CDC"), "Class should be CDC");
        assert!(
            fields[7].decoded.contains("0483"),
            "idVendor should be 0x0483"
        );
    }

    #[test]
    fn test_detect_usb_bot_frame() {
        let cbw_sig = [0x43, 0x42, 0x53, 0x55]; // USBC in LE
        let mut cbw = vec![0u8; 31];
        cbw[..4].copy_from_slice(&cbw_sig);
        assert_eq!(detect_usb_bot_frame(&cbw), Some("CBW"));

        let csw_sig = [0x53, 0x42, 0x53, 0x55]; // USBS in LE
        let mut csw = vec![0u8; 13];
        csw[..4].copy_from_slice(&csw_sig);
        assert_eq!(detect_usb_bot_frame(&csw), Some("CSW"));

        assert_eq!(detect_usb_bot_frame(&[0x00, 0x00, 0x00, 0x00]), None);
    }

    #[test]
    fn test_usb_class_request_name_dispatch() {
        assert_eq!(
            usb_class_request_name(UsbClassHint::Standard, 0x06),
            "GET_DESCRIPTOR"
        );
        assert_eq!(
            usb_class_request_name(UsbClassHint::CdcAcm, 0x20),
            "SET_LINE_CODING"
        );
        assert_eq!(
            usb_class_request_name(UsbClassHint::Hid, 0x01),
            "GET_REPORT"
        );
        assert_eq!(
            usb_class_request_name(UsbClassHint::MassStorage, 0xFE),
            "GET_MAX_LUN"
        );
        assert_eq!(usb_class_request_name(UsbClassHint::Audio, 0x01), "SET_CUR");
        assert_eq!(usb_class_request_name(UsbClassHint::Video, 0x81), "GET_CUR");
    }

    #[test]
    fn test_usb_transfer_type_names() {
        assert_eq!(usb_transfer_type_name(0x00), "Control");
        assert_eq!(usb_transfer_type_name(0x01), "Isochronous");
        assert_eq!(usb_transfer_type_name(0x02), "Bulk");
        assert_eq!(usb_transfer_type_name(0x03), "Interrupt");
    }

    #[test]
    fn test_scsi_opcode_names() {
        assert_eq!(scsi_opcode_name(0x00), "TEST_UNIT_READY");
        assert_eq!(scsi_opcode_name(0x12), "INQUIRY");
        assert_eq!(scsi_opcode_name(0x28), "READ(10)");
        assert_eq!(scsi_opcode_name(0x2A), "WRITE(10)");
        assert_eq!(scsi_opcode_name(0x25), "READ_CAPACITY(10)");
    }

    #[test]
    fn test_usb_checks_cdc_line_coding() {
        // CDC SET_LINE_CODING 115200 8N1
        let mut pkt = vec![0x21, 0x20, 0x00, 0x00, 0x00, 0x00, 0x07, 0x00];
        pkt.extend_from_slice(&115200u32.to_le_bytes());
        pkt.extend_from_slice(&[0x00, 0x00, 0x08]);
        let kpi = IndustrialKpi {
            payload_utilization_pct: 50.0,
            frame_error_rate_pct: 0.0,
            bit_error_rate_ppm: 0.0,
            avg_inter_frame_ms: 1.0,
            jitter_ms: 0.5,
            duplicate_payload_ratio_pct: 0.0,
        };
        let checks = usb_checks(&pkt, &kpi);
        assert!(checks
            .iter()
            .any(|c| c.name == "USB Class" && c.detail.contains("CDC")));
        assert!(checks
            .iter()
            .any(|c| c.name == "CDC line coding len" && c.pass));
        assert!(checks
            .iter()
            .any(|c| c.name == "Baud rate validity" && c.pass));
    }

    #[test]
    fn test_usb_checks_bot_cbw() {
        let mut cbw = vec![0u8; 31];
        cbw[0..4].copy_from_slice(&[0x43, 0x42, 0x53, 0x55]);
        cbw[14] = 0x06; // cbLen = 6
        let kpi = IndustrialKpi {
            payload_utilization_pct: 50.0,
            frame_error_rate_pct: 0.0,
            bit_error_rate_ppm: 0.0,
            avg_inter_frame_ms: 1.0,
            jitter_ms: 0.5,
            duplicate_payload_ratio_pct: 0.0,
        };
        let checks = usb_checks(&cbw, &kpi);
        assert!(checks.iter().any(|c| c.name == "CBW signature" && c.pass));
        assert!(checks.iter().any(|c| c.name == "CBW LUN range" && c.pass));
        assert!(checks.iter().any(|c| c.name == "CBW CB length" && c.pass));
    }

    #[test]
    fn test_usb_cs_interface_names() {
        assert_eq!(usb_cs_interface_name(0x00), "Header");
        assert_eq!(usb_cs_interface_name(0x02), "ACM");
        assert_eq!(usb_cs_interface_name(0x06), "Union");
        assert_eq!(usb_cs_interface_name(0x0F), "Ethernet Networking");
    }

    #[test]
    fn test_detect_usb_descriptor_type() {
        // Valid device descriptor header (18 bytes minimum)
        let mut dev_desc = vec![0u8; 18];
        dev_desc[0] = 18;
        dev_desc[1] = 1;
        assert_eq!(detect_usb_descriptor_type(&dev_desc), Some(1));
        // Valid config descriptor header (9 bytes)
        assert_eq!(
            detect_usb_descriptor_type(&[9, 2, 32, 0, 1, 1, 0, 0x80, 50]),
            Some(2)
        );
        // Invalid: bLength = 0
        assert_eq!(detect_usb_descriptor_type(&[0, 1, 0, 2]), None);
        // Invalid: bLength > data len
        assert_eq!(detect_usb_descriptor_type(&[50, 1]), None);
        // Partial header too short
        assert_eq!(detect_usb_descriptor_type(&[18, 1, 0, 2]), None);
    }
}


