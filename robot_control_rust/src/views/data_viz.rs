use crate::app::AppState;
use crate::i18n::Tr;
use crate::models::data_channel::{DataSource, RobotStateField, VizType};
use crate::views::ui_kit::{page_header, settings_card};
use egui::{self, Color32, RichText, Ui};
use egui_plot::{Bar, BarChart, Line, Plot, PlotPoints, Points};

pub fn show(ui: &mut Ui, state: &mut AppState) {
    let lang = state.lang();
    page_header(ui, Tr::tab_data_viz(lang), "viz");

    // ─── 通道管理面板 ────────────────────────────────────
    settings_card(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            ui.label(RichText::new(format!("{}:", Tr::channels(lang))).strong());

            // 逐个通道显示启用复选框 + 类型指示 + 来源
            for (i, ch) in state.data_channels.iter_mut().enumerate() {
                let source_text = match &ch.source {
                    DataSource::RobotState(field) => format!("Robot/{}", field),
                    DataSource::PacketField {
                        template_name,
                        field_name,
                    } => format!("Packet/{}/{}", template_name, field_name),
                    DataSource::RawOffset { offset, .. } => format!("Raw@{}", offset),
                };
                let label = format!("{} ({}, {})", ch.name, ch.viz_type, source_text);
                let _ = i;
                ui.checkbox(&mut ch.enabled, &label);
            }
        });
    });

    ui.add_space(10.0);

    // ─── 添加/删除通道 ──────────────────────────────────
    settings_card(ui, |ui| {
        ui.collapsing(Tr::viz_channel_config(lang), |ui| {
            // 现有通道的可视化类型修改
            let mut to_remove = None;
            for (i, ch) in state.data_channels.iter_mut().enumerate() {
                ui.horizontal_wrapped(|ui| {
                    ui.label(format!("{}:", ch.name));
                    egui::ComboBox::from_id_salt(format!("viz_type_{}", i))
                        .selected_text(format!("{}", ch.viz_type))
                        .width(130.0)
                        .show_ui(ui, |ui| {
                            for &vt in VizType::all() {
                                ui.selectable_value(&mut ch.viz_type, vt, format!("{}", vt));
                            }
                        });

                    // 颜色预览
                    let [r, g, b] = ch.color;
                    ui.colored_label(Color32::from_rgb(r, g, b), "■");

                    if ui
                        .small_button(Tr::delete(lang))
                        .on_hover_text(Tr::delete(lang))
                        .clicked()
                    {
                        to_remove = Some(i);
                    }
                });
            }
            if let Some(idx) = to_remove {
                state.data_channels.remove(idx);
                if idx < state.channel_buffers.len() {
                    state.channel_buffers.remove(idx);
                }
            }

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(4.0);

            // 添加新通道：选择来源 + 类型
            ui.horizontal_wrapped(|ui| {
                ui.label(Tr::viz_add_channel(lang));
                ui.add(
                    egui::TextEdit::singleline(&mut state.ui.viz_add_channel_name)
                        .hint_text(Tr::name(lang))
                        .desired_width(100.0),
                );

                let source_types = ["RobotState", "PacketField"];
                egui::ComboBox::from_id_salt("viz_source_type")
                    .selected_text(
                        *source_types
                            .get(state.ui.viz_source_type)
                            .unwrap_or(&"RobotState"),
                    )
                    .width(110.0)
                    .show_ui(ui, |ui| {
                        for (i, label) in source_types.iter().enumerate() {
                            ui.selectable_value(&mut state.ui.viz_source_type, i, *label);
                        }
                    });

                let packet_fields = state.available_packet_fields();
                if state.ui.viz_source_type == 0 {
                    let sources = [
                        "Position",
                        "Velocity",
                        "Current",
                        "Temperature",
                        "Error",
                        "PID Output",
                    ];
                    egui::ComboBox::from_id_salt("viz_add_src")
                        .selected_text(*sources.get(state.ui.viz_add_source_idx).unwrap_or(&"?"))
                        .width(130.0)
                        .show_ui(ui, |ui| {
                            for (i, &s) in sources.iter().enumerate() {
                                ui.selectable_value(&mut state.ui.viz_add_source_idx, i, s);
                            }
                        });
                } else if packet_fields.is_empty() {
                    ui.label(RichText::new("No parsed numeric fields").color(Color32::GRAY));
                } else {
                    if state.ui.viz_pkt_template_idx >= packet_fields.len() {
                        state.ui.viz_pkt_template_idx = 0;
                    }
                    let (tpl, fld) = &packet_fields[state.ui.viz_pkt_template_idx];
                    egui::ComboBox::from_id_salt("viz_pkt_field")
                        .selected_text(format!("{}/{}", tpl, fld))
                        .width(220.0)
                        .show_ui(ui, |ui| {
                            for (i, (t, f)) in packet_fields.iter().enumerate() {
                                ui.selectable_value(
                                    &mut state.ui.viz_pkt_template_idx,
                                    i,
                                    format!("{}/{}", t, f),
                                );
                            }
                        });
                }

                let viz_types = VizType::all();
                let vt = viz_types
                    .get(state.ui.viz_add_type_idx)
                    .copied()
                    .unwrap_or(VizType::Line);
                egui::ComboBox::from_id_salt("viz_add_type")
                    .selected_text(format!("{}", vt))
                    .width(120.0)
                    .show_ui(ui, |ui| {
                        for (i, &v) in viz_types.iter().enumerate() {
                            ui.selectable_value(
                                &mut state.ui.viz_add_type_idx,
                                i,
                                format!("{}", v),
                            );
                        }
                    });

                if ui.button(Tr::add_field(lang)).clicked()
                    && !state.ui.viz_add_channel_name.is_empty()
                {
                    let colors = [
                        [65, 155, 255],
                        [255, 165, 0],
                        [255, 100, 100],
                        [255, 100, 255],
                        [255, 50, 50],
                        [100, 255, 100],
                        [200, 200, 50],
                        [100, 200, 200],
                    ];
                    let c = colors[state.data_channels.len() % colors.len()];
                    use crate::models::data_channel::DataChannel;
                    let source = if state.ui.viz_source_type == 0 {
                        let field = match state.ui.viz_add_source_idx {
                            0 => RobotStateField::Position,
                            1 => RobotStateField::Velocity,
                            2 => RobotStateField::Current,
                            3 => RobotStateField::Temperature,
                            4 => RobotStateField::Error,
                            _ => RobotStateField::PidOutput,
                        };
                        DataSource::RobotState(field)
                    } else {
                        if packet_fields.is_empty() {
                            state.status_message =
                                "No parsed numeric packet field available".into();
                            return;
                        }
                        let idx = state.ui.viz_pkt_template_idx.min(packet_fields.len() - 1);
                        let (template_name, field_name) = &packet_fields[idx];
                        DataSource::PacketField {
                            template_name: template_name.clone(),
                            field_name: field_name.clone(),
                        }
                    };
                    let ch = DataChannel::new(&state.ui.viz_add_channel_name, source, vt, c);
                    state.data_channels.push(ch);
                    state
                        .channel_buffers
                        .push(crate::models::data_channel::TimeSeriesBuffer::default());
                    state.ui.viz_add_channel_name.clear();
                }
            });
        });
    });

    ui.add_space(10.0);

    // ─── 数据统计 ────────────────────────────────────────
    settings_card(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.label(format!(
                "{}: {} / 2000",
                Tr::data_points(lang),
                state.state_history.len()
            ));
            if state.channel_overflow_events > 0 {
                ui.separator();
                ui.label(
                    RichText::new(format!("Dropped points: {}", state.channel_overflow_events))
                        .color(Color32::from_rgb(255, 180, 120)),
                );
            }
            ui.separator();
            if ui.button(Tr::clear_history(lang)).clicked() {
                state.state_history.clear();
                for buf in &mut state.channel_buffers {
                    buf.clear();
                }
                state.status_message = Tr::clear_history(lang).into();
            }
        });
    });

    ui.add_space(10.0);

    if state.state_history.is_empty() && state.channel_buffers.iter().all(|b| b.data.is_empty()) {
        ui.add_space(24.0);
        ui.label(
            RichText::new(Tr::no_data_hint(lang))
                .size(14.0)
                .color(Color32::GRAY)
                .italics(),
        );
        return;
    }

    // ─── 更新通道缓冲区 ─────────────────────────────────
    // (Sync RobotState-based channels from state_history)
    while state.channel_buffers.len() < state.data_channels.len() {
        state
            .channel_buffers
            .push(crate::models::data_channel::TimeSeriesBuffer::default());
    }
    let mut dropped_total = 0usize;
    for (i, ch) in state.data_channels.iter().enumerate() {
        if let DataSource::RobotState(field) = &ch.source {
            let buf = &mut state.channel_buffers[i];
            let expected_len = state.state_history.len();
            if buf.data.len() < expected_len {
                let start = buf.data.len();
                for s in &state.state_history[start..] {
                    let v = match field {
                        RobotStateField::Position => s.position,
                        RobotStateField::Velocity => s.velocity,
                        RobotStateField::Current => s.current,
                        RobotStateField::Temperature => s.temperature,
                        RobotStateField::Error => s.error,
                        RobotStateField::PidOutput => s.pid_output,
                    };
                    let dropped = buf.push_with_overflow(v);
                    if dropped > 0 {
                        dropped_total += dropped;
                    }
                }
            }
        }
    }
    if dropped_total > 0 {
        state.report_channel_overflow(dropped_total);
    }

    // ─── 渲染各通道 ─────────────────────────────────────
    // Split channels by viz type for efficient rendering
    let enabled_channels: Vec<(usize, &crate::models::data_channel::DataChannel)> = state
        .data_channels
        .iter()
        .enumerate()
        .filter(|(_, ch)| ch.enabled)
        .collect();

    // Group: Line/Scatter together in one plot, others separately
    let line_scatter: Vec<_> = enabled_channels
        .iter()
        .filter(|(_, ch)| matches!(ch.viz_type, VizType::Line | VizType::Scatter))
        .collect();
    let bars: Vec<_> = enabled_channels
        .iter()
        .filter(|(_, ch)| ch.viz_type == VizType::Bar)
        .collect();
    let gauges: Vec<_> = enabled_channels
        .iter()
        .filter(|(_, ch)| ch.viz_type == VizType::Gauge)
        .collect();
    let histograms: Vec<_> = enabled_channels
        .iter()
        .filter(|(_, ch)| ch.viz_type == VizType::Histogram)
        .collect();
    let tables: Vec<_> = enabled_channels
        .iter()
        .filter(|(_, ch)| ch.viz_type == VizType::Table)
        .collect();

    let remaining_height = ui.available_height() - 24.0;
    let section_count = [
        !line_scatter.is_empty(),
        !bars.is_empty(),
        !gauges.is_empty(),
        !histograms.is_empty(),
        !tables.is_empty(),
    ]
    .iter()
    .filter(|&&b| b)
    .count()
    .max(1);
    let section_height = (remaining_height / section_count as f32).max(140.0);

    // ─── Line / Scatter 图表 ─────────────────────────────
    if !line_scatter.is_empty() {
        Plot::new("viz_line_scatter_plot")
            .height(section_height)
            .legend(egui_plot::Legend::default())
            .show_axes(true)
            .show(ui, |plot_ui| {
                for &&(idx, ch) in &line_scatter {
                    let [r, g, b] = ch.color;
                    let color = Color32::from_rgb(r, g, b);
                    if idx < state.channel_buffers.len() {
                        let pts = state.channel_buffers[idx].as_plot_points();
                        let plot_pts: PlotPoints = pts.into_iter().collect();
                        match ch.viz_type {
                            VizType::Line => {
                                plot_ui.line(
                                    Line::new(plot_pts).name(&ch.name).color(color).width(1.5),
                                );
                            }
                            VizType::Scatter => {
                                plot_ui.points(
                                    Points::new(plot_pts)
                                        .name(&ch.name)
                                        .color(color)
                                        .radius(2.0),
                                );
                            }
                            _ => {}
                        }
                    }
                }
            });
        ui.add_space(4.0);
    }

    // ─── Bar 图表 ────────────────────────────────────────
    if !bars.is_empty() {
        Plot::new("viz_bar_plot")
            .height(section_height)
            .legend(egui_plot::Legend::default())
            .show(ui, |plot_ui| {
                for (bar_i, &&(idx, ch)) in bars.iter().enumerate() {
                    let [r, g, b] = ch.color;
                    let color = Color32::from_rgb(r, g, b);
                    if idx < state.channel_buffers.len() {
                        let data = state.channel_buffers[idx].last_n(50);
                        let bar_items: Vec<Bar> = data
                            .iter()
                            .enumerate()
                            .map(|(i, &v)| {
                                Bar::new(i as f64 + bar_i as f64 * 0.3, v)
                                    .width(0.25)
                                    .fill(color)
                            })
                            .collect();
                        plot_ui.bar_chart(BarChart::new(bar_items).name(&ch.name).color(color));
                    }
                }
            });
        ui.add_space(4.0);
    }

    // ─── Gauge 仪表盘 ───────────────────────────────────
    if !gauges.is_empty() {
        ui.horizontal_wrapped(|ui| {
            for &&(idx, ch) in &gauges {
                if idx < state.channel_buffers.len() {
                    let stats = state.channel_buffers[idx].statistics();
                    let [r, g, b] = ch.color;
                    let color = Color32::from_rgb(r, g, b);
                    egui::Frame::group(ui.style())
                        .inner_margin(egui::Margin::same(10))
                        .show(ui, |ui| {
                            ui.set_min_width(120.0);
                            ui.vertical_centered(|ui| {
                                ui.label(RichText::new(&ch.name).strong().size(13.0));
                                ui.add_space(4.0);
                                ui.label(
                                    RichText::new(format!("{:.2}", stats.last))
                                        .size(28.0)
                                        .color(color),
                                );
                                ui.add_space(2.0);
                                if !ch.unit.is_empty() {
                                    ui.label(
                                        RichText::new(&ch.unit).size(11.0).color(Color32::GRAY),
                                    );
                                }
                                ui.add_space(4.0);
                                ui.label(
                                    RichText::new(format!(
                                        "min:{:.1} max:{:.1} avg:{:.1}",
                                        stats.min, stats.max, stats.mean
                                    ))
                                    .size(10.0)
                                    .color(Color32::GRAY),
                                );
                            });
                        });
                }
            }
        });
        ui.add_space(4.0);
    }

    // ─── Histogram 直方图 ────────────────────────────────
    if !histograms.is_empty() {
        Plot::new("viz_histogram_plot")
            .height(section_height)
            .legend(egui_plot::Legend::default())
            .show(ui, |plot_ui| {
                for &&(idx, ch) in &histograms {
                    let [r, g, b] = ch.color;
                    let color = Color32::from_rgb(r, g, b);
                    if idx < state.channel_buffers.len() {
                        let hist = state.channel_buffers[idx].histogram(20);
                        let bars: Vec<Bar> = hist
                            .iter()
                            .map(|&(center, count)| {
                                Bar::new(center, count as f64).width(0.8).fill(color)
                            })
                            .collect();
                        plot_ui.bar_chart(BarChart::new(bars).name(&ch.name).color(color));
                    }
                }
            });
        ui.add_space(4.0);
    }

    // ─── Table 表格 ──────────────────────────────────────
    if !tables.is_empty() {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            egui::ScrollArea::horizontal().show(ui, |ui| {
                egui::Grid::new("viz_table")
                    .num_columns(7)
                    .spacing([12.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        // Header
                        ui.label(RichText::new(Tr::name(lang)).strong());
                        ui.label(RichText::new("Last").strong());
                        ui.label(RichText::new("Min").strong());
                        ui.label(RichText::new("Max").strong());
                        ui.label(RichText::new("Mean").strong());
                        ui.label(RichText::new("StdDev").strong());
                        ui.label(RichText::new("Count").strong());
                        ui.end_row();

                        for &&(idx, ch) in &tables {
                            if idx < state.channel_buffers.len() {
                                let stats = state.channel_buffers[idx].statistics();
                                let [r, g, b] = ch.color;
                                ui.colored_label(Color32::from_rgb(r, g, b), &ch.name);
                                ui.label(format!("{:.4}", stats.last));
                                ui.label(format!("{:.4}", stats.min));
                                ui.label(format!("{:.4}", stats.max));
                                ui.label(format!("{:.4}", stats.mean));
                                ui.label(format!("{:.4}", stats.std_dev));
                                ui.label(format!("{}", stats.count));
                                ui.end_row();
                            }
                        }
                    });
            });
        });
    }
}
