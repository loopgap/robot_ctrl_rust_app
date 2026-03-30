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

pub fn load_preferences() -> Result<AppPreferences, String> {
    let path = prefs_path();

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read preferences file {}: {}", path.display(), e))?;

    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse preferences: {}", e))
        .or_else(|_| {
            eprintln!("Warning: Preferences file corrupted, using defaults");
            Ok(AppPreferences::default())
        })
}
pub fn save_preferences(prefs: &AppPreferences) -> Result<(), String> {
    let path = prefs_path();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    let json = serde_json::to_string_pretty(prefs)
        .map_err(|e| format!("Failed to serialize preferences: {}", e))?;

    fs::write(&path, json)
        .map_err(|e| format!("Failed to write preferences file {}: {}", path.display(), e))
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
