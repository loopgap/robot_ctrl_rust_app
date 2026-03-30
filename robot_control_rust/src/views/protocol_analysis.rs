use crate::app::AppState;
use egui::Ui;
use std::path::PathBuf;

#[derive(Default)]
pub struct ProtocolAnalyzer {}

pub fn show(ui: &mut Ui, state: &mut AppState) {
    ui.heading("Protocol Analyzer");
}

pub fn export_analysis_csv(state: &AppState) -> Result<PathBuf, String> {
    Ok(PathBuf::from("export.csv"))
}
