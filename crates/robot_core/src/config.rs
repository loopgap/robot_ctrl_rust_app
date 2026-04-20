//! Configuration management for plugins

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[cfg(feature = "serde")]
#[derive(Serialize, Deserialize, Default, Clone)]
#[cfg_attr(not(feature = "serde"), derive(Default, Clone))]
pub struct PluginConfig {
    pub enabled: Vec<String>,
    pub disabled: Vec<String>,
}

#[cfg(not(feature = "serde"))]
#[derive(Default, Clone)]
pub struct PluginConfig {
    pub enabled: Vec<String>,
    pub disabled: Vec<String>,
}

impl PluginConfig {
    #[cfg(feature = "serde")]
    pub fn load(name: &str) -> anyhow::Result<Self> {
        let path = Self::path(name)?;
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(Self::default())
        }
    }

    #[cfg(not(feature = "serde"))]
    pub fn load(_name: &str) -> anyhow::Result<Self> {
        Ok(Self::default())
    }

    #[cfg(feature = "serde")]
    pub fn save(&self, name: &str) -> anyhow::Result<()> {
        let path = Self::path(name)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }

    #[cfg(not(feature = "serde"))]
    pub fn save(&self, name: &str) -> anyhow::Result<()> {
        let path = Self::path(name)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = format!(
            r#"{{"enabled": {:?}, "disabled": {:?}}}"#,
            self.enabled, self.disabled
        );
        std::fs::write(&path, content)?;
        Ok(())
    }

    fn path(app_name: &str) -> anyhow::Result<PathBuf> {
        let base = if cfg!(target_os = "windows") {
            std::env::var("APPDATA")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("."))
        } else if cfg!(target_os = "linux") {
            std::env::var("HOME")
                .map(|h| PathBuf::from(h).join(".config"))
                .unwrap_or_else(|_| PathBuf::from("."))
        } else {
            PathBuf::from(".")
        };

        Ok(base.join(app_name).join("plugins.json"))
    }

    pub fn is_enabled(&self, plugin_name: &str) -> bool {
        if self.disabled.contains(&plugin_name.to_string()) {
            return false;
        }
        if self.enabled.is_empty() {
            return true;
        }
        self.enabled.contains(&plugin_name.to_string())
    }

    pub fn enable(&mut self, plugin_name: &str) {
        if !self.enabled.contains(&plugin_name.to_string()) {
            self.enabled.push(plugin_name.to_string());
        }
        self.disabled.retain(|d| d != plugin_name);
    }

    pub fn disable(&mut self, plugin_name: &str) {
        if !self.disabled.contains(&plugin_name.to_string()) {
            self.disabled.push(plugin_name.to_string());
        }
        self.enabled.retain(|e| e != plugin_name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_config_default() {
        let config = PluginConfig::default();
        assert!(config.enabled.is_empty());
        assert!(config.disabled.is_empty());
    }

    #[test]
    fn test_enable_disable() {
        let mut config = PluginConfig::default();
        config.enable("modbus");
        assert!(config.is_enabled("modbus"));
        config.disable("modbus");
        assert!(!config.is_enabled("modbus"));
    }
}
