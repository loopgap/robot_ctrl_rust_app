use crate::app::{AppState, LogDirection};
use crate::i18n::{Language, Tr};
use crate::views::ui_kit::{page_header, section_title, settings_card};
use egui::{Color32, RichText, Ui};
use std::path::PathBuf;

#[derive(Default)]
pub struct ProtocolAnalyzer {}

fn tr_filters(lang: Language) -> &'static str {
    match lang {
        Language::English => "Filters",
        Language::Chinese => "筛选条件",
    }
}

fn tr_search(lang: Language) -> &'static str {
    match lang {
        Language::English => "Search",
        Language::Chinese => "搜索",
    }
}

fn tr_search_hint(lang: Language) -> &'static str {
    match lang {
        Language::English => "Keyword: channel/data/timestamp",
        Language::Chinese => "关键词: 通道/数据/时间",
    }
}

fn tr_results(lang: Language) -> &'static str {
    match lang {
        Language::English => "Analysis Results",
        Language::Chinese => "分析结果",
    }
}

fn tr_no_results(lang: Language) -> &'static str {
    match lang {
        Language::English => "No matching logs.",
        Language::Chinese => "未匹配到日志。",
    }
}

fn tr_export_success(path: &std::path::Path, lang: Language) -> String {
    match lang {
        Language::English => format!("Analysis exported: {}", path.display()),
        Language::Chinese => format!("分析结果已导出: {}", path.display()),
    }
}

fn tr_export_failed(err: &str, lang: Language) -> String {
    match lang {
        Language::English => format!("Export failed: {}", err),
        Language::Chinese => format!("导出失败: {}", err),
    }
}

fn direction_label(direction: LogDirection) -> &'static str {
    match direction {
        LogDirection::Tx => "TX",
        LogDirection::Rx => "RX",
        LogDirection::Info => "INFO",
    }
}

fn direction_color(direction: LogDirection) -> Color32 {
    match direction {
        LogDirection::Tx => Color32::from_rgb(120, 200, 255),
        LogDirection::Rx => Color32::from_rgb(130, 230, 160),
        LogDirection::Info => Color32::from_rgb(220, 220, 140),
    }
}

fn include_direction(state: &AppState, direction: LogDirection) -> bool {
    match direction {
        LogDirection::Tx => state.ui.analysis_filter_tx,
        LogDirection::Rx => state.ui.analysis_filter_rx,
        LogDirection::Info => state.ui.analysis_filter_info,
    }
}

fn contains_query(state: &AppState, haystack: &str) -> bool {
    let query = state.ui.analysis_query.trim();
    if query.is_empty() {
        return true;
    }
    haystack.to_lowercase().contains(&query.to_lowercase())
}

pub fn show(ui: &mut Ui, state: &mut AppState) {
    let lang = state.lang();
    page_header(ui, Tr::tab_protocol_analysis(lang), "packet");

    settings_card(ui, |ui| {
        section_title(ui, tr_filters(lang));
        ui.horizontal_wrapped(|ui| {
            ui.checkbox(&mut state.ui.analysis_filter_tx, "TX");
            ui.checkbox(&mut state.ui.analysis_filter_rx, "RX");
            ui.checkbox(&mut state.ui.analysis_filter_info, "INFO");
        });

        ui.add_space(6.0);
        ui.label(tr_search(lang));
        ui.text_edit_singleline(&mut state.ui.analysis_query)
            .on_hover_text(tr_search_hint(lang));

        ui.add_space(8.0);
        if ui.button(Tr::menu_export_log(lang)).clicked() {
            match export_analysis_csv(state) {
                Ok(path) => state.status_message = tr_export_success(&path, lang),
                Err(e) => state.status_message = tr_export_failed(&e, lang),
            }
        }
    });

    ui.add_space(10.0);
    settings_card(ui, |ui| {
        section_title(ui, tr_results(lang));

        let filtered: Vec<_> = state
            .log_entries
            .iter()
            .rev()
            .filter(|entry| include_direction(state, entry.direction))
            .filter(|entry| {
                let text = format!(
                    "{} {} {} {}",
                    entry.timestamp,
                    entry.channel,
                    direction_label(entry.direction),
                    entry.format_data()
                );
                contains_query(state, &text)
            })
            .take(500)
            .collect();

        if filtered.is_empty() {
            ui.label(RichText::new(tr_no_results(lang)).color(Color32::GRAY));
            return;
        }

        egui::ScrollArea::vertical()
            .max_height(420.0)
            .show(ui, |ui| {
                for entry in filtered {
                    let line = format!(
                        "[{}] [{}] [{}] {}",
                        entry.timestamp,
                        entry.channel,
                        direction_label(entry.direction),
                        entry.format_data()
                    );
                    ui.label(
                        RichText::new(line)
                            .monospace()
                            .color(direction_color(entry.direction)),
                    );
                }
            });
    });
}

fn csv_escape(value: &str) -> String {
    let escaped = value.replace('"', "\"\"");
    format!("\"{}\"", escaped)
}

pub fn export_analysis_csv(state: &AppState) -> Result<PathBuf, String> {
    let mut export_dir = AppState::user_prefs_path();
    export_dir.pop();
    export_dir.push("exports");
    std::fs::create_dir_all(&export_dir).map_err(|e| format!("create export dir failed: {}", e))?;

    let file_name = format!(
        "protocol_analysis_{}.csv",
        chrono::Local::now().format("%Y%m%d_%H%M%S")
    );
    let file_path = export_dir.join(file_name);

    let mut csv = String::from("timestamp,channel,direction,display_mode,data\n");
    for entry in &state.log_entries {
        let direction = direction_label(entry.direction);
        let mode = match entry.display_mode {
            crate::app::DisplayMode::Hex => "HEX",
            crate::app::DisplayMode::Ascii => "ASCII",
            crate::app::DisplayMode::Mixed => "MIXED",
        };
        csv.push_str(&format!(
            "{},{},{},{},{}\n",
            csv_escape(&entry.timestamp),
            csv_escape(&entry.channel),
            csv_escape(direction),
            csv_escape(mode),
            csv_escape(&entry.format_data())
        ));
    }

    std::fs::write(&file_path, csv).map_err(|e| format!("write export file failed: {}", e))?;
    Ok(file_path)
}
