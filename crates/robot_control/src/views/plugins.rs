//! Plugin Settings View
//!
//! Provides UI for managing plugin configuration with restart prompts.

use robot_core::PluginConfig;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct PluginSettings {
    pub config: PluginConfig,
    pub pending_restart: bool,
}

impl std::fmt::Debug for PluginSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginSettings")
            .field("pending_restart", &self.pending_restart)
            .finish()
    }
}

#[allow(clippy::derivable_impls)]
impl Default for PluginSettings {
    fn default() -> Self {
        Self {
            config: PluginConfig::default(),
            pending_restart: false,
        }
    }
}

impl PluginSettings {
    pub fn load() -> Self {
        let config = PluginConfig::load("robot_control").unwrap_or_default();
        Self {
            config,
            pending_restart: false,
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        self.config.save("robot_control")
    }

    pub fn toggle_plugin(&mut self, plugin_name: &str) {
        if self.config.is_enabled(plugin_name) {
            self.config.disable(plugin_name);
        } else {
            self.config.enable(plugin_name);
        }
        self.pending_restart = true;
        let _ = self.save();
    }

    pub fn dismiss_restart_prompt(&mut self) {
        self.pending_restart = false;
    }
}

pub struct PluginListItem {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub default_enabled: bool,
}

impl PluginListItem {
    pub fn all() -> Vec<Self> {
        vec![
            PluginListItem {
                name: "modbus".to_string(),
                display_name: "Modbus".to_string(),
                description: "Modbus RTU/TCP protocol support".to_string(),
                default_enabled: true,
            },
            PluginListItem {
                name: "canopen".to_string(),
                display_name: "CANopen".to_string(),
                description: "CANopen protocol support".to_string(),
                default_enabled: true,
            },
            PluginListItem {
                name: "llm".to_string(),
                display_name: "LLM Integration".to_string(),
                description: "AI-powered PID tuning suggestions".to_string(),
                default_enabled: true,
            },
            PluginListItem {
                name: "nn".to_string(),
                display_name: "Neural Network".to_string(),
                description: "Neural network-based PID tuning".to_string(),
                default_enabled: false,
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_settings_default() {
        let settings = PluginSettings::default();
        assert!(!settings.pending_restart);
    }

    #[test]
    fn test_toggle_plugin_sets_pending_restart() {
        let mut settings = PluginSettings::default();
        settings.toggle_plugin("modbus");
        assert!(settings.pending_restart);
    }
}
