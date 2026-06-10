use crate::app::{ActiveTab, AppState};
use crate::i18n::{Language, Tr};
use crate::services::ConnectionProvider;
use crate::views::ui_kit::{page_header, section_title, settings_card};
use egui::{self, Color32, RichText, Ui, Vec2};

pub fn show(ui: &mut Ui, state: &mut AppState) {
    let theme = state.theme.clone();
    let current_time = ui.ctx().input(|i| i.time);
    let lang = state.lang();
    page_header(ui, Tr::tab_dashboard(lang), "dashboard");

    // ═══ 连接状态卡片 ═══════════════════════════════════
    settings_card(ui, |ui| {
        section_title(ui, Tr::connection_status(lang));
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(10.0, 10.0);
            connection_card(
                ui,
                "Serial",
                &state.conn.serial.status.to_string(),
                status_color(state.conn.serial.is_connected()),
            );
            connection_card(
                ui,
                "TCP",
                &state.conn.tcp.status.to_string(),
                status_color(state.conn.tcp.is_connected()),
            );
            connection_card(
                ui,
                "UDP",
                &state.conn.udp.status.to_string(),
                status_color(state.conn.udp.is_connected()),
            );
            connection_card(
                ui,
                "CAN",
                if state.conn.can.is_running {
                    "Running"
                } else {
                    "Stopped"
                },
                status_color(state.conn.can.is_running),
            );
        });
    });

    ui.add_space(10.0);

    // ═══ 启动自检 ═══════════════════════════════════════
    settings_card(ui, |ui| {
        section_title(ui, Tr::system_check_label(lang));
        let (ok_count, total_count) = state.system_check_summary();
        ui.label(
            RichText::new(if lang == Language::Chinese {
                format!(
                    "通过: {}/{}  |  版本: {}",
                    ok_count, total_count, state.build_version
                )
            } else {
                format!(
                    "Passed: {}/{}  |  Version: {}",
                    ok_count, total_count, state.build_version
                )
            })
            .color(if ok_count == total_count {
                theme.status_ok
            } else {
                theme.status_warn
            }),
        );
        ui.label(
            RichText::new(state.update_status_summary())
                .size(12.0)
                .color(if state.update_available {
                    theme.status_warn
                } else {
                    theme.text_label
                }),
        );
        ui.label(
            RichText::new(if lang == Language::Chinese {
                format!(
                    "更新: {} | 上次检查: {}",
                    state.update_status_detail, state.update_last_checked_at
                )
            } else {
                format!(
                    "Update: {} | Last check: {}",
                    state.update_status_detail, state.update_last_checked_at
                )
            })
            .size(11.5),
        );
        ui.add_space(6.0);
        for check in state.system_checks.iter().take(6) {
            let icon = if check.ok { "OK" } else { "WARN" };
            ui.label(
                RichText::new(format!("{} {} - {}", icon, check.name, check.detail)).size(12.0),
            );
        }
    });

    ui.add_space(10.0);

    // ═══ 系统统计 ═══════════════════════════════════════
    settings_card(ui, |ui| {
        section_title(ui, Tr::system_stats(lang));
        egui::Grid::new("stats_grid")
            .num_columns(2)
            .spacing([28.0, 8.0])
            .show(ui, |ui| {
                stat_row(
                    ui,
                    Tr::bytes_sent(lang),
                    &format_bytes(state.total_bytes_sent()),
                );
                ui.end_row();
                stat_row(
                    ui,
                    Tr::bytes_received(lang),
                    &format_bytes(state.total_bytes_received()),
                );
                ui.end_row();
                stat_row(
                    ui,
                    Tr::total_errors(lang),
                    &state.total_errors().to_string(),
                );
                ui.end_row();
                stat_row(
                    ui,
                    Tr::log_entries(lang),
                    &state.log.log_entries.len().to_string(),
                );
                ui.end_row();
                stat_row(
                    ui,
                    Tr::state_history(lang),
                    &state.control.state_history.len().to_string(),
                );
                ui.end_row();
                stat_row(
                    ui,
                    Tr::active_channel(lang),
                    &state.conn.active_conn.to_string(),
                );
                ui.end_row();
                stat_row(ui, Tr::last_comm(lang), state.last_comm());
                ui.end_row();
            });
    });

    ui.add_space(10.0);

    // ═══ 快捷操作 ═══════════════════════════════════════
    settings_card(ui, |ui| {
        section_title(ui, Tr::quick_actions(lang));
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 12.0;

            if state.is_any_connected() {
                if ui
                    .button(RichText::new(Tr::disconnect(lang)).size(14.0))
                    .clicked()
                {
                    state.disconnect_active();
                    state.status_message = Tr::disconnected(lang).into();
                }
            } else if ui
                .button(RichText::new(Tr::connect(lang)).size(14.0))
                .clicked()
            {
                match state.connect_active() {
                    Ok(()) => {
                        state.status_message = if state.active_status().is_connected() {
                            Tr::connected(lang).into()
                        } else {
                            if lang == Language::Chinese {
                                "连接中..."
                            } else {
                                "Connecting..."
                            }
                            .into()
                        }
                    }
                    Err(e) => state.report_error(format!("{}: {}", Tr::error_label(lang), e)),
                }
            }

            let run_text = if state.control.is_running {
                RichText::new(Tr::stop_control(lang))
                    .size(14.0)
                    .color(state.anim.animate_color(
                        "dashboard_1".into(),
                        theme.status_error,
                        theme.status_error,
                        0.3,
                        crate::app::animation::Easing::EaseOut,
                        current_time,
                    ))
            } else {
                RichText::new(Tr::start_control(lang))
                    .size(14.0)
                    .color(state.anim.animate_color(
                        "dashboard_1".into(),
                        theme.status_ok,
                        theme.status_ok,
                        0.3,
                        crate::app::animation::Easing::EaseOut,
                        current_time,
                    ))
            };
            if ui.button(run_text).clicked() {
                state.toggle_running();
            }

            if ui
                .button(
                    RichText::new(Tr::emergency_stop(lang))
                        .size(14.0)
                        .color(Color32::RED)
                        .strong(),
                )
                .clicked()
            {
                state.emergency_stop();
            }

            if ui
                .button(RichText::new(Tr::refresh_ports(lang)).size(14.0))
                .clicked()
            {
                state.refresh_ports();
                state.status_message = Tr::found_ports(state.conn.available_ports.len(), lang);
            }

            let update_text = if state.update_available {
                "⬆ Open Available Update"
            } else {
                "⬆ Check Updates"
            };
            if ui.button(RichText::new(update_text).size(14.0)).clicked() {
                let url = state.trigger_update_check();
                ui.ctx().open_url(egui::OpenUrl { url, new_tab: true });
            }
        });
    });

    ui.add_space(10.0);

    // ═══ 机器人状态 ═══════════════════════════════════════
    settings_card(ui, |ui| {
        section_title(ui, Tr::robot_state(lang));
        let s = &state.control.current_state;
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(18.0, 10.0);
            state_cell(ui, Tr::position(lang), &format!("{:.2}", s.position));
            state_cell(ui, Tr::velocity(lang), &format!("{:.2}", s.velocity));
            state_cell(ui, Tr::current_a(lang), &format!("{:.2} A", s.current));
            state_cell(
                ui,
                Tr::temperature(lang),
                &format!("{:.1} \u{00B0}C", s.temperature),
            );
            state_cell(ui, Tr::error_ch(lang), &format!("{:.3}", s.error));
            state_cell(ui, Tr::pid_output(lang), &format!("{:.2}", s.pid_output));
            state_cell(
                ui,
                if lang == Language::Chinese {
                    "电压"
                } else {
                    "Voltage"
                },
                &format!("{:.1} V", s.voltage),
            );
            state_cell(
                ui,
                if lang == Language::Chinese {
                    "PWM 占空比"
                } else {
                    "PWM"
                },
                &format!("{:.1}%", s.pwm_duty),
            );
        });
    });

    // ═══ 拓扑信息 ════════════════════════════════════════
    ui.add_space(10.0);

    // ═══ 运行指标 ════════════════════════════════════════
    settings_card(ui, |ui| {
        section_title(ui, Tr::runtime_metrics_label(lang));
        let (mcp_req, mcp_unauth) = state.mcp_metrics_snapshot();
        egui::Grid::new("runtime_metrics_grid")
            .num_columns(2)
            .spacing([28.0, 8.0])
            .show(ui, |ui| {
                stat_row(
                    ui,
                    if lang == Language::Chinese {
                        "连接尝试次数"
                    } else {
                        "Connect Attempts"
                    },
                    &state.metrics.connect_attempts.to_string(),
                );
                ui.end_row();
                stat_row(
                    ui,
                    if lang == Language::Chinese {
                        "连接失败次数"
                    } else {
                        "Connect Failures"
                    },
                    &state.metrics.connect_failures.to_string(),
                );
                ui.end_row();
                stat_row(
                    ui,
                    if lang == Language::Chinese {
                        "LLM 请求数"
                    } else {
                        "LLM Requests"
                    },
                    &state.metrics.llm_requests.to_string(),
                );
                ui.end_row();
                stat_row(
                    ui,
                    if lang == Language::Chinese {
                        "LLM 成功数"
                    } else {
                        "LLM Success"
                    },
                    &state.metrics.llm_success.to_string(),
                );
                ui.end_row();
                stat_row(
                    ui,
                    if lang == Language::Chinese {
                        "LLM 失败数"
                    } else {
                        "LLM Failures"
                    },
                    &state.metrics.llm_failures.to_string(),
                );
                ui.end_row();
                stat_row(
                    ui,
                    if lang == Language::Chinese {
                        "MCP 启动次数"
                    } else {
                        "MCP Startups"
                    },
                    &state.metrics.mcp_startups.to_string(),
                );
                ui.end_row();
                stat_row(
                    ui,
                    if lang == Language::Chinese {
                        "MCP 请求数"
                    } else {
                        "MCP Requests"
                    },
                    &mcp_req.to_string(),
                );
                ui.end_row();
                stat_row(
                    ui,
                    if lang == Language::Chinese {
                        "MCP 未授权次数"
                    } else {
                        "MCP Unauthorized"
                    },
                    &mcp_unauth.to_string(),
                );
                ui.end_row();
            });
    });

    ui.add_space(10.0);
    settings_card(ui, |ui| {
        section_title(ui, Tr::topology_info(lang));
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 16.0;
            ui.label(format!("{}", state.control.topology.chassis_type));
            ui.label(format!(
                "| {} {}",
                state.control.topology.motors.len(),
                Tr::motors(lang)
            ));
            ui.label(format!(
                "| PID: Kp={:.3} Ki={:.3} Kd={:.3}",
                state.control.pid().kp,
                state.control.pid().ki,
                state.control.pid().kd
            ));
        });
    });

    ui.add_space(10.0);

    settings_card(ui, |ui| {
        section_title(ui, Tr::protocol_analysis_entry_label(lang));
        let (tx, rx, info) = state.log.counts();

        ui.horizontal_wrapped(|ui| {
            ui.label(format!(
                "{}: {}",
                if lang == Language::Chinese {
                    "总帧数"
                } else {
                    "Total Frames"
                },
                state.log.log_entries.len()
            ));
            ui.separator();
            ui.label(format!("TX: {}", tx));
            ui.label(format!("RX: {}", rx));
            ui.label(format!("INFO: {}", info));
        });

        ui.add_space(6.0);
        ui.label(if lang == Language::Chinese {
            "协议分析已集成在主界面中，可直接在侧边栏“协议分析”页使用完整分析工具。"
        } else {
            "Protocol analysis is integrated into the main workspace. Open the Protocol Analysis tab for the full toolset."
        });
        if ui
            .button(if lang == Language::Chinese {
                "打开协议分析页"
            } else {
                "Open Protocol Analysis"
            })
            .clicked()
        {
            state.active_tab = ActiveTab::ProtocolAnalysis;
        }
    });
}

// ─── 辅助函数 ─────────────────────────────────────────────

fn connection_card(ui: &mut Ui, label: &str, status: &str, color: Color32) {
    egui::Frame::new()
        .fill(Color32::from_rgba_premultiplied(50, 50, 60, 180))
        .corner_radius(6.0)
        .inner_margin(12.0)
        .show(ui, |ui| {
            ui.set_min_size(Vec2::new(165.0, 0.0));
            ui.label(RichText::new(label).size(13.0).strong());
            ui.add_space(4.0);
            ui.label(RichText::new(status).size(12.0).color(color));
        });
}

pub(crate) fn status_color(connected: bool) -> Color32 {
    if connected {
        Color32::from_rgb(46, 160, 67)
    } else {
        Color32::from_rgb(128, 128, 128)
    }
}

fn stat_row(ui: &mut Ui, label: &str, value: &str) {
    ui.label(RichText::new(label).size(13.0).color(Color32::GRAY));
    ui.label(RichText::new(value).size(13.0).strong());
}

fn state_cell(ui: &mut Ui, label: &str, value: &str) {
    ui.vertical(|ui| {
        ui.label(RichText::new(label).size(11.5).color(Color32::GRAY));
        ui.add_space(2.0);
        ui.label(RichText::new(value).size(15.0).strong());
    });
}

pub(crate) fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        return format!("{} B", bytes);
    }
    if bytes < 1024 * 1024 {
        return format!("{:.1} KB", bytes as f64 / 1024.0);
    }
    format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes_zero() {
        assert_eq!(format_bytes(0), "0 B");
    }

    #[test]
    fn test_format_bytes_one() {
        assert_eq!(format_bytes(1), "1 B");
    }

    #[test]
    fn test_format_bytes_1023() {
        assert_eq!(format_bytes(1023), "1023 B");
    }

    #[test]
    fn test_format_bytes_1kb() {
        assert_eq!(format_bytes(1024), "1.0 KB");
    }

    #[test]
    fn test_format_bytes_1mb() {
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
    }

    #[test]
    fn test_format_bytes_1gb() {
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1024.00 MB");
    }

    #[test]
    fn test_status_color_connected() {
        assert_eq!(status_color(true), Color32::from_rgb(46, 160, 67));
    }

    #[test]
    fn test_status_color_disconnected() {
        assert_eq!(status_color(false), Color32::from_rgb(128, 128, 128));
    }
}
