use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Language {
    #[default]
    Zh,
    En,
}

impl Language {
    pub fn label(self) -> &'static str {
        match self {
            Self::Zh => "中文",
            Self::En => "English",
        }
    }

    pub fn tr(self, zh: &str, en: &str) -> String {
        match self {
            Self::Zh => zh.to_string(),
            Self::En => en.to_string(),
        }
    }
}
