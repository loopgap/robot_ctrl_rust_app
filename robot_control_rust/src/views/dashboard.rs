use crate::app::{ActiveTab, AppState};
use crate::i18n::{Language, Tr};
use crate::services::ConnectionProvider;
use crate::views::ui_kit::{page_header, section_title, settings_card};
use egui::{self, Color32, RichText, Ui, Vec2};

pub fn show(ui: &mut Ui, state: &mut AppState) {
    let theme = state.theme.clone();
    let current_time = ui.ctx().input(|i| i.time);
    let lang = state.lang();

    // ── 1) RichText: styled page header ──────────────────────
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
                if lang == Language::Chinese {
                    "串口连接状态"
                } else {
                    "Serial port connection"
                },
            );
            connection_card(
                ui,
                "TCP",
                &state.conn.tcp.status.to_string(),
                status_color(state.conn.tcp.is_connected()),
                if lang == Language::Chinese {
                    "TCP/IP 网络连接状态"
                } else {
                    "TCP/IP network connection"
                },
            );
            connection_card(
                ui,
                "UDP",
                &state.conn.udp.status.to_string(),
                status_color(state.conn.udp.is_connected()),
                if lang == Language::Chinese {
                    "UDP 数据报连接状态"
                } else {
                    "UDP datagram connection"
                },
            );
            connection_card(
                ui,
                "CAN",
                if state.conn.can.is_running {
                    Tr::running_label(lang)
                } else {
                    Tr::stopped_label(lang)
                },
                status_color(state.conn.can.is_running),
                if lang == Language::Chinese {
                    "CAN/CAN FD 总线状态"
                } else {
                    "CAN / CAN FD bus status"
                },
            );
        });

        // ── 3) ProgressBar: overall connection health ────────
        ui.add_space(6.0);
        let connected_count = [
            state.conn.serial.is_connected(),
            state.conn.tcp.is_connected(),
            state.conn.udp.is_connected(),
            state.conn.can.is_running,
        ]
        .iter()
        .filter(|&&c| c)
        .count();
        let health_ratio = connected_count as f32 / 4.0;
        let health_color = if health_ratio > 0.5 {
            theme.status_ok
        } else if health_ratio > 0.0 {
            theme.status_warn
        } else {
            Color32::from_rgb(128, 128, 128)
        };
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(Tr::top_health(lang))
                    .size(12.0)
                    .color(theme.text_label),
            );
            let health_bar = egui::ProgressBar::new(health_ratio)
                .fill(health_color)
                .text(format!("{}/4", connected_count));
            // 2) Tooltip: on_hover_text on health bar
            ui.add(health_bar)
                .on_hover_text(if lang == Language::Chinese {
                    "已连接通道数 / 总通道数"
                } else {
                    "Connected channels / Total channels"
                });
        });
    });

    // ── 4) Separator ─────────────────────────────────────────
    ui.add_space(6.0);
    ui.separator();
    ui.add_space(4.0);

    // ═══ 启动自检 ═══════════════════════════════════════
    settings_card(ui, |ui| {
        section_title(ui, Tr::system_check_label(lang));
        let (ok_count, total_count) = state.system_check_summary();

        // ── 3) ProgressBar: system check pass rate ───────────
        let check_ratio = if total_count > 0 {
            ok_count as f32 / total_count as f32
        } else {
            0.0
        };
        let check_color = if check_ratio >= 1.0 {
            theme.status_ok
        } else if check_ratio >= 0.5 {
            theme.status_warn
        } else {
            theme.status_error
        };
        ui.horizontal(|ui| {
            // ── 2) RichText: styled label ────────────────────
            ui.label(
                RichText::new(if lang == Language::Chinese {
                    "自检结果"
                } else {
                    "Check Result"
                })
                .size(13.0)
                .strong()
                .color(theme.text_label),
            );
            let check_bar = egui::ProgressBar::new(check_ratio)
                .fill(check_color)
                .text(format!("{}/{}", ok_count, total_count));
            // ── 1) Tooltip ───────────────────────────────────
            ui.add(check_bar)
                .on_hover_text(if lang == Language::Chinese {
                    "所有自检项的通过率"
                } else {
                    "Pass rate of all system checks"
                });
        });

        // ── 2) RichText: version & status ────────────────────
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

        // ── 5) ScrollArea + 6) CollapsingHeader: system checks detail ──
        egui::CollapsingHeader::new(
            RichText::new(if lang == Language::Chinese {
                "自检详情"
            } else {
                "Check Details"
            })
            .size(13.0)
            .strong(),
        )
        .default_open(false)
        .show(ui, |ui| {
            egui::ScrollArea::vertical()
                .max_height(160.0)
                .show(ui, |ui| {
                    for check in state.system_checks.iter() {
                        let icon = if check.ok { "OK" } else { "WARN" };
                        let icon_color = if check.ok {
                            theme.status_ok
                        } else {
                            theme.status_warn
                        };
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(icon).size(12.0).color(icon_color));
                            ui.label(
                                RichText::new(format!("{} - {}", check.name, check.detail))
                                    .size(12.0),
                            );
                        });
                    }
                });
        });
    });

    // ── 4) Separator ─────────────────────────────────────────
    ui.add_space(6.0);
    ui.separator();
    ui.add_space(4.0);

    // ═══ 系统统计 ═══════════════════════════════════════
    settings_card(ui, |ui| {
        section_title(ui, Tr::system_stats(lang));
        egui::Grid::new("stats_grid")
            .num_columns(2)
            .spacing([28.0, 8.0])
            .show(ui, |ui| {
                // ── 1) Tooltip: on hover for each stat ───────
                stat_row_with_tooltip(
                    ui,
                    Tr::bytes_sent(lang),
                    &format_bytes(state.total_bytes_sent()),
                    if lang == Language::Chinese {
                        "自启动以来已发送的数据总量"
                    } else {
                        "Total data sent since application start"
                    },
                );
                ui.end_row();
                stat_row_with_tooltip(
                    ui,
                    Tr::bytes_received(lang),
                    &format_bytes(state.total_bytes_received()),
                    if lang == Language::Chinese {
                        "自启动以来已接收的数据总量"
                    } else {
                        "Total data received since application start"
                    },
                );
                ui.end_row();
                stat_row_with_tooltip(
                    ui,
                    Tr::total_errors(lang),
                    &state.total_errors().to_string(),
                    if lang == Language::Chinese {
                        "所有通道累计错误次数"
                    } else {
                        "Cumulative error count across all channels"
                    },
                );
                ui.end_row();
                stat_row_with_tooltip(
                    ui,
                    Tr::log_entries(lang),
                    &state.log.log_entries.len().to_string(),
                    if lang == Language::Chinese {
                        "通信日志中记录的条目数量"
                    } else {
                        "Number of entries in the communication log"
                    },
                );
                ui.end_row();
                stat_row_with_tooltip(
                    ui,
                    Tr::state_history(lang),
                    &state.control.state_history.len().to_string(),
                    if lang == Language::Chinese {
                        "控制器状态历史记录数量"
                    } else {
                        "Number of controller state history records"
                    },
                );
                ui.end_row();
                stat_row_with_tooltip(
                    ui,
                    Tr::active_channel(lang),
                    &state.conn.active_conn.to_string(),
                    if lang == Language::Chinese {
                        "当前选中的活动通信通道"
                    } else {
                        "Currently selected active communication channel"
                    },
                );
                ui.end_row();
                stat_row_with_tooltip(
                    ui,
                    Tr::last_comm(lang),
                    state.last_comm(),
                    if lang == Language::Chinese {
                        "最后一次成功通信的时间戳"
                    } else {
                        "Timestamp of the last successful communication"
                    },
                );
                ui.end_row();
            });
    });

    // ── 4) Separator ─────────────────────────────────────────
    ui.add_space(6.0);
    ui.separator();
    ui.add_space(4.0);

    // ═══ 快捷操作 ═══════════════════════════════════════
    settings_card(ui, |ui| {
        section_title(ui, Tr::quick_actions(lang));
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 12.0;

            if state.is_any_connected() {
                let btn = ui
                    .button(RichText::new(Tr::disconnect(lang)).size(14.0))
                    // ── 1) Tooltip ───────────────────────────
                    .on_hover_text(if lang == Language::Chinese {
                        "断开当前活动通道的连接"
                    } else {
                        "Disconnect the currently active channel"
                    });
                if btn.clicked() {
                    state.disconnect_active();
                    state.status_message = Tr::disconnected(lang).into();
                }
            } else {
                let btn = ui
                    .button(RichText::new(Tr::connect(lang)).size(14.0))
                    .on_hover_text(if lang == Language::Chinese {
                        "连接当前活动通道"
                    } else {
                        "Connect to the currently active channel"
                    });
                if btn.clicked() {
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
            let run_btn = ui
                .button(run_text)
                .on_hover_text(if lang == Language::Chinese {
                    "启动或停止控制回路"
                } else {
                    "Start or stop the control loop"
                });
            if run_btn.clicked() {
                state.toggle_running();
            }

            // ── 1) Tooltip: emergency stop ───────────────────
            let estop_btn = ui
                .button(
                    RichText::new(Tr::emergency_stop(lang))
                        .size(14.0)
                        .color(Color32::RED)
                        .strong(),
                )
                .on_hover_text(if lang == Language::Chinese {
                    "立即停止所有电机输出（安全操作）"
                } else {
                    "Immediately halt all motor output (safety operation)"
                });
            if estop_btn.clicked() {
                state.emergency_stop();
            }

            // ── 1) Tooltip: refresh ports ────────────────────
            let refresh_btn = ui
                .button(RichText::new(Tr::refresh_ports(lang)).size(14.0))
                .on_hover_text(if lang == Language::Chinese {
                    "重新扫描可用串口设备"
                } else {
                    "Re-scan for available serial port devices"
                });
            if refresh_btn.clicked() {
                state.refresh_ports();
                state.status_message = Tr::found_ports(state.conn.available_ports.len(), lang);
            }

            let update_text = if state.update_available {
                "⬆ Open Available Update"
            } else {
                "⬆ Check Updates"
            };
            // ── 1) Tooltip: update check ─────────────────────
            let update_btn = ui
                .button(RichText::new(update_text).size(14.0))
                .on_hover_text(if lang == Language::Chinese {
                    "检查软件更新或打开更新页面"
                } else {
                    "Check for software updates or open the update page"
                });
            if update_btn.clicked() {
                let url = state.trigger_update_check();
                ui.ctx().open_url(egui::OpenUrl { url, new_tab: true });
            }
        });
    });

    // ── 4) Separator ─────────────────────────────────────────
    ui.add_space(6.0);
    ui.separator();
    ui.add_space(4.0);

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

        // ── 3) ProgressBar: visual gauges for key metrics ────
        ui.add_space(8.0);
        egui::Grid::new("robot_state_gauges")
            .num_columns(2)
            .spacing([20.0, 6.0])
            .show(ui, |ui| {
                // Temperature gauge (0–120°C typical range)
                ui.label(
                    RichText::new(Tr::temperature(lang))
                        .size(12.0)
                        .color(theme.text_label),
                );
                let temp_ratio = (s.temperature as f32 / 120.0).clamp(0.0, 1.0);
                let temp_color = if temp_ratio < 0.5 {
                    theme.status_ok
                } else if temp_ratio < 0.75 {
                    theme.status_warn
                } else {
                    theme.status_error
                };
                let temp_bar = egui::ProgressBar::new(temp_ratio)
                    .fill(temp_color)
                    .text(format!("{:.1}\u{00B0}C", s.temperature));
                ui.add(temp_bar)
                    .on_hover_text(if lang == Language::Chinese {
                        "温度范围 0–120°C，超过 90°C 请注意散热"
                    } else {
                        "Temperature range 0-120°C, ensure cooling above 90°C"
                    });
                ui.end_row();

                // PWM duty gauge (0–100%)
                ui.label(
                    RichText::new(if lang == Language::Chinese {
                        "PWM 占空比"
                    } else {
                        "PWM Duty"
                    })
                    .size(12.0)
                    .color(theme.text_label),
                );
                let pwm_ratio = (s.pwm_duty as f32 / 100.0).clamp(0.0, 1.0);
                let pwm_color = if pwm_ratio < 0.8 {
                    theme.accent_blue
                } else {
                    theme.status_warn
                };
                let pwm_bar = egui::ProgressBar::new(pwm_ratio)
                    .fill(pwm_color)
                    .text(format!("{:.1}%", s.pwm_duty));
                ui.add(pwm_bar).on_hover_text(if lang == Language::Chinese {
                    "PWM 输出占空比 (0–100%)"
                } else {
                    "PWM output duty cycle (0-100%)"
                });
                ui.end_row();

                // Voltage gauge (typical 0–48V range)
                ui.label(
                    RichText::new(if lang == Language::Chinese {
                        "电压"
                    } else {
                        "Voltage"
                    })
                    .size(12.0)
                    .color(theme.text_label),
                );
                let volt_ratio = (s.voltage as f32 / 48.0).clamp(0.0, 1.0);
                let volt_color = if volt_ratio > 0.15 && volt_ratio < 0.85 {
                    theme.status_ok
                } else {
                    theme.status_warn
                };
                let volt_bar = egui::ProgressBar::new(volt_ratio)
                    .fill(volt_color)
                    .text(format!("{:.1} V", s.voltage));
                ui.add(volt_bar)
                    .on_hover_text(if lang == Language::Chinese {
                        "母线电压 (典型范围 0–48V)"
                    } else {
                        "Bus voltage (typical range 0-48V)"
                    });
                ui.end_row();
            });
    });

    // ── 4) Separator ─────────────────────────────────────────
    ui.add_space(6.0);
    ui.separator();
    ui.add_space(4.0);

    // ═══ 6) CollapsingHeader: 运行指标 ════════════════════
    settings_card(ui, |ui| {
        egui::CollapsingHeader::new(
            RichText::new(Tr::runtime_metrics_label(lang))
                .size(17.0)
                .strong(),
        )
        .default_open(true)
        .show(ui, |ui| {
            let (mcp_req, mcp_unauth) = state.mcp_metrics_snapshot();
            egui::Grid::new("runtime_metrics_grid")
                .num_columns(2)
                .spacing([28.0, 8.0])
                .show(ui, |ui| {
                    stat_row_with_tooltip(
                        ui,
                        if lang == Language::Chinese {
                            "连接尝试次数"
                        } else {
                            "Connect Attempts"
                        },
                        &state.metrics.connect_attempts.to_string(),
                        if lang == Language::Chinese {
                            "应用启动后尝试建立连接的总次数"
                        } else {
                            "Total connection attempts since app start"
                        },
                    );
                    ui.end_row();
                    stat_row_with_tooltip(
                        ui,
                        if lang == Language::Chinese {
                            "连接失败次数"
                        } else {
                            "Connect Failures"
                        },
                        &state.metrics.connect_failures.to_string(),
                        if lang == Language::Chinese {
                            "连接尝试失败的次数（超时、拒绝等）"
                        } else {
                            "Number of failed connection attempts (timeout, refused, etc.)"
                        },
                    );
                    ui.end_row();
                    stat_row_with_tooltip(
                        ui,
                        if lang == Language::Chinese {
                            "LLM 请求数"
                        } else {
                            "LLM Requests"
                        },
                        &state.metrics.llm_requests.to_string(),
                        if lang == Language::Chinese {
                            "向 LLM 服务发送的请求总数"
                        } else {
                            "Total requests sent to the LLM service"
                        },
                    );
                    ui.end_row();
                    stat_row_with_tooltip(
                        ui,
                        if lang == Language::Chinese {
                            "LLM 成功数"
                        } else {
                            "LLM Success"
                        },
                        &state.metrics.llm_success.to_string(),
                        if lang == Language::Chinese {
                            "LLM 请求成功返回结果的次数"
                        } else {
                            "Number of LLM requests that returned successfully"
                        },
                    );
                    ui.end_row();
                    stat_row_with_tooltip(
                        ui,
                        if lang == Language::Chinese {
                            "LLM 失败数"
                        } else {
                            "LLM Failures"
                        },
                        &state.metrics.llm_failures.to_string(),
                        if lang == Language::Chinese {
                            "LLM 请求失败的次数（网络错误、超时等）"
                        } else {
                            "Number of failed LLM requests (network error, timeout, etc.)"
                        },
                    );
                    ui.end_row();
                    stat_row_with_tooltip(
                        ui,
                        if lang == Language::Chinese {
                            "MCP 启动次数"
                        } else {
                            "MCP Startups"
                        },
                        &state.metrics.mcp_startups.to_string(),
                        if lang == Language::Chinese {
                            "MCP 服务器被启动的次数"
                        } else {
                            "Number of times the MCP server has been started"
                        },
                    );
                    ui.end_row();
                    stat_row_with_tooltip(
                        ui,
                        if lang == Language::Chinese {
                            "MCP 请求数"
                        } else {
                            "MCP Requests"
                        },
                        &mcp_req.to_string(),
                        if lang == Language::Chinese {
                            "MCP 服务器接收到的请求总数"
                        } else {
                            "Total requests received by the MCP server"
                        },
                    );
                    ui.end_row();
                    stat_row_with_tooltip(
                        ui,
                        if lang == Language::Chinese {
                            "MCP 未授权次数"
                        } else {
                            "MCP Unauthorized"
                        },
                        &mcp_unauth.to_string(),
                        if lang == Language::Chinese {
                            "因认证失败被拒绝的 MCP 请求次数"
                        } else {
                            "Number of MCP requests rejected due to auth failure"
                        },
                    );
                    ui.end_row();
                });

            // ── 3) ProgressBar: LLM success rate ─────────────
            if state.metrics.llm_requests > 0 {
                ui.add_space(6.0);
                let llm_ratio =
                    state.metrics.llm_success as f32 / state.metrics.llm_requests as f32;
                let llm_color = if llm_ratio >= 0.8 {
                    theme.status_ok
                } else if llm_ratio >= 0.5 {
                    theme.status_warn
                } else {
                    theme.status_error
                };
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(if lang == Language::Chinese {
                            "LLM 成功率"
                        } else {
                            "LLM Success Rate"
                        })
                        .size(12.0)
                        .color(theme.text_label),
                    );
                    let llm_bar = egui::ProgressBar::new(llm_ratio)
                        .fill(llm_color)
                        .text(format!("{:.0}%", llm_ratio * 100.0));
                    ui.add(llm_bar).on_hover_text(if lang == Language::Chinese {
                        "LLM 请求成功率 = 成功数 / 总请求数"
                    } else {
                        "LLM request success rate = successes / total requests"
                    });
                });
            }
        });
    });

    // ── 4) Separator ─────────────────────────────────────────
    ui.add_space(6.0);
    ui.separator();
    ui.add_space(4.0);

    // ═══ 6) CollapsingHeader: 拓扑信息 ════════════════════
    settings_card(ui, |ui| {
        egui::CollapsingHeader::new(RichText::new(Tr::topology_info(lang)).size(17.0).strong())
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing.x = 16.0;
                    ui.label(
                        RichText::new(format!("{}", state.control.topology.chassis_type)).strong(),
                    )
                    .on_hover_text(if lang == Language::Chinese {
                        "当前机器人底盘运动学类型"
                    } else {
                        "Current robot chassis kinematics type"
                    });
                    ui.separator();
                    ui.label(RichText::new(format!(
                        "{} {}",
                        state.control.topology.motors.len(),
                        Tr::motors(lang)
                    )))
                    .on_hover_text(if lang == Language::Chinese {
                        "配置的电机/关节数量"
                    } else {
                        "Number of configured motors/joints"
                    });
                    ui.separator();
                    ui.label(RichText::new(format!(
                        "PID: Kp={:.3} Ki={:.3} Kd={:.3}",
                        state.control.pid().kp,
                        state.control.pid().ki,
                        state.control.pid().kd
                    )))
                    .on_hover_text(if lang == Language::Chinese {
                        "当前 PID 控制器增益参数"
                    } else {
                        "Current PID controller gain parameters"
                    });
                });
            });
    });

    // ── 4) Separator ─────────────────────────────────────────
    ui.add_space(6.0);
    ui.separator();
    ui.add_space(4.0);

    // ═══ 6) CollapsingHeader: 协议分析入口 ════════════════════
    settings_card(ui, |ui| {
        egui::CollapsingHeader::new(
            RichText::new(Tr::protocol_analysis_entry_label(lang))
                .size(17.0)
                .strong(),
        )
        .default_open(true)
        .show(ui, |ui| {
            let (tx, rx, info) = state.log.counts();

            ui.horizontal_wrapped(|ui| {
                ui.label(
                    RichText::new(format!(
                        "{}: {}",
                        if lang == Language::Chinese {
                            "总帧数"
                        } else {
                            "Total Frames"
                        },
                        state.log.log_entries.len()
                    ))
                    .strong(),
                )
                .on_hover_text(if lang == Language::Chinese {
                    "已记录的通信帧总数"
                } else {
                    "Total number of recorded communication frames"
                });
                ui.separator();
                ui.label(
                    RichText::new(format!("TX: {}", tx))
                        .color(theme.tx_color),
                )
                .on_hover_text(if lang == Language::Chinese {
                    "发送帧数量"
                } else {
                    "Transmitted frame count"
                });
                ui.label(
                    RichText::new(format!("RX: {}", rx))
                        .color(theme.rx_color),
                )
                .on_hover_text(if lang == Language::Chinese {
                    "接收帧数量"
                } else {
                    "Received frame count"
                });
                ui.label(
                    RichText::new(format!("INFO: {}", info))
                        .color(theme.info_color),
                )
                .on_hover_text(if lang == Language::Chinese {
                    "信息帧数量"
                } else {
                    "Information frame count"
                });
            });

            // ── 3) ProgressBar: TX / RX ratio ────────────────
            let total_frames = tx + rx;
            if total_frames > 0 {
                ui.add_space(4.0);
                let tx_ratio = tx as f32 / total_frames as f32;
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("TX/RX")
                            .size(12.0)
                            .color(theme.text_label),
                    );
                    let tx_bar = egui::ProgressBar::new(tx_ratio)
                        .fill(theme.tx_color)
                        .text(format!("TX {:.0}% / RX {:.0}%", tx_ratio * 100.0, (1.0 - tx_ratio) * 100.0));
                    ui.add(tx_bar).on_hover_text(if lang == Language::Chinese {
                        "发送帧与接收帧的比例"
                    } else {
                        "Ratio of transmitted to received frames"
                    });
                });
            }

            ui.add_space(6.0);
            ui.label(if lang == Language::Chinese {
                "协议分析已集成在主界面中，可直接在侧边栏\"协议分析\"页使用完整分析工具。"
            } else {
                "Protocol analysis is integrated into the main workspace. Open the Protocol Analysis tab for the full toolset."
            });

            // ── 1) Tooltip: protocol analysis button ─────────
            if ui
                .button(if lang == Language::Chinese {
                    "打开协议分析页"
                } else {
                    "Open Protocol Analysis"
                })
                .on_hover_text(if lang == Language::Chinese {
                    "切换到协议分析标签页，查看完整的协议分析工具"
                } else {
                    "Switch to the Protocol Analysis tab for the full toolset"
                })
                .clicked()
            {
                state.active_tab = ActiveTab::ProtocolAnalysis;
            }
        });
    });
}

// ─── 辅助函数 ─────────────────────────────────────────────

fn connection_card(ui: &mut Ui, label: &str, status: &str, color: Color32, tooltip: &str) {
    let frame = egui::Frame::new()
        .fill(Color32::from_rgba_premultiplied(50, 50, 60, 180))
        .corner_radius(6.0)
        .inner_margin(12.0)
        .show(ui, |ui| {
            ui.set_min_size(Vec2::new(165.0, 0.0));
            // ── 2) RichText: styled connection label ─────────
            ui.label(RichText::new(label).size(13.0).strong());
            ui.add_space(4.0);
            ui.label(RichText::new(status).size(12.0).color(color));
        });
    // ── 1) Tooltip: on_hover_text for connection card frame ──
    frame.response.on_hover_text(tooltip);
}

pub(crate) fn status_color(connected: bool) -> Color32 {
    if connected {
        Color32::from_rgb(46, 160, 67)
    } else {
        Color32::from_rgb(128, 128, 128)
    }
}

/// stat_row with tooltip — extends original with hover hint
fn stat_row_with_tooltip(ui: &mut Ui, label: &str, value: &str, tooltip: &str) {
    ui.label(RichText::new(label).size(13.0).color(Color32::GRAY));
    ui.label(RichText::new(value).size(13.0).strong())
        .on_hover_text(tooltip);
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
