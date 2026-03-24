use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::i18n::Language;

#[derive(Clone, Serialize, Deserialize)]
pub struct AppPreferences {
    pub language: Language,
}

impl Default for AppPreferences {
    fn default() -> Self {
        Self {
            language: Language::Zh,
        }
    }
}

pub fn load_preferences() -> AppPreferences {
    let path = prefs_path();
    let content = match fs::read_to_string(path) {
        Ok(v) => v,
        Err(_) => return AppPreferences::default(),
    };

    serde_json::from_str(&content).unwrap_or_default()
}

pub fn save_preferences(prefs: &AppPreferences) {
    let path = prefs_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(prefs) {
        let _ = fs::write(path, json);
    }
}

fn prefs_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            return PathBuf::from(appdata)
                .join("rust_micro_tools")
                .join("preferences.json");
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("rust_micro_tools")
                .join("preferences.json");
        }
    }

    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home)
            .join(".config")
            .join("rust_micro_tools")
            .join("preferences.json");
    }

    PathBuf::from("preferences.json")
}
