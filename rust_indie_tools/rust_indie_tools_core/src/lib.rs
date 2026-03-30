use eframe::egui;

pub const ACCENT: egui::Color32 = egui::Color32::from_rgb(0, 122, 204);

#[derive(Default, Clone)]
pub struct WorkflowState {
    pub input_provided: bool,
    pub executed: bool,
    pub verified: bool,
    pub exported: bool,
    pub detail: String,
}

impl WorkflowState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        self.input_provided = false;
        self.executed = false;
        self.verified = false;
        self.exported = false;
        self.detail.clear();
    }

    pub fn mark_executed(&mut self) {
        self.executed = true;
        self.exported = false;
    }

    pub fn mark_verified(&mut self, detail: &str) {
        self.verified = true;
        self.detail = detail.to_string();
    }

    pub fn mark_failed(&mut self, detail: &str) {
        self.verified = false;
        self.detail = detail.to_string();
    }

    pub fn mark_exported(&mut self) {
        self.exported = true;
    }
}

pub struct WorkflowStep<'a> {
    pub name: &'a str,
    pub done: bool,
    pub detail: &'a str,
}

pub fn render_workflow_panel(ui: &mut egui::Ui, steps: &[WorkflowStep<'_>]) {
    ui.heading("Workflow");
    ui.separator();

    for step in steps {
        let icon = if step.done { "●" } else { "○" };
        let color = if step.done {
            egui::Color32::LIGHT_GREEN
        } else {
            egui::Color32::GRAY
        };

        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.colored_label(color, icon);
                ui.strong(step.name);
            });
            ui.label(step.detail);
        });
        ui.add_space(4.0);
    }
}

pub fn render_action_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add(egui::Button::new(label).fill(ACCENT))
}

pub fn render_export_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.button(label)
}

pub fn render_text_area(ui: &mut egui::Ui, text: &mut String, height: f32) {
    ui.add_sized(
        [ui.available_width(), height],
        egui::TextEdit::multiline(text),
    );
}

pub fn render_header_panel(ctx: &egui::Context, title: &str) {
    egui::TopBottomPanel::top("header").show(ctx, |ui| {
        ui.heading(title);
        ui.colored_label(
            ACCENT,
            "Pipeline: Input -> Validate -> Execute -> Verify -> Export",
        );
    });
}

pub fn apply_dark_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.override_text_color = Some(egui::Color32::from_rgb(220, 220, 230));
    ctx.set_visuals(visuals);
}
