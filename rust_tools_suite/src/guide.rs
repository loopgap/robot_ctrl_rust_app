use crate::i18n::Language;
use eframe::egui;

pub fn render_guide(
    ui: &mut egui::Ui,
    lang: Language,
    title_zh: &str,
    title_en: &str,
    lines: &[(&str, &str)],
) {
    let title = match lang {
        Language::Zh => format!("使用引导 · {title_zh}"),
        Language::En => format!("Guide · {title_en}"),
    };

    egui::CollapsingHeader::new(title)
        .default_open(true)
        .show(ui, |ui| {
            for (index, (zh, en)) in lines.iter().enumerate() {
                let text = match lang {
                    Language::Zh => *zh,
                    Language::En => *en,
                };
                ui.label(format!("{}. {text}", index + 1));
            }
        });
}
