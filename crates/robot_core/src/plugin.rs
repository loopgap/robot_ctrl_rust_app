//! Plugin system for robot applications

use crate::error::{Error, Result};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Core trait that all plugins must implement
pub trait Plugin: Send + Sync {
    /// Unique name of the plugin
    fn name(&self) -> &str;

    /// Version string
    fn version(&self) -> &str;

    /// Whether this plugin is currently enabled
    fn is_enabled(&self) -> bool;

    /// Enable the plugin
    fn enable(&mut self) -> Result<()>;

    /// Disable the plugin
    fn disable(&mut self) -> Result<()>;
}

/// Registry for managing plugins at runtime
pub struct PluginRegistry {
    plugins: RwLock<HashMap<String, Arc<dyn Plugin>>>,
    enabled: RwLock<Vec<String>>,
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: RwLock::new(HashMap::new()),
            enabled: RwLock::new(Vec::new()),
        }
    }

    /// Register a new plugin
    pub fn register(&self, plugin: Arc<dyn Plugin>) -> Result<()> {
        let name = plugin.name().to_string();
        let mut plugins = self
            .plugins
            .write()
            .map_err(|_| Error::Plugin("Lock poisoned".into()))?;
        plugins.insert(name.clone(), plugin);
        Ok(())
    }

    /// Unregister a plugin by name
    pub fn unregister(&self, name: &str) -> Result<()> {
        let mut plugins = self
            .plugins
            .write()
            .map_err(|_| Error::Plugin("Lock poisoned".into()))?;
        let mut enabled = self
            .enabled
            .write()
            .map_err(|_| Error::Plugin("Lock poisoned".into()))?;
        plugins.remove(name);
        enabled.retain(|n| n != name);
        Ok(())
    }

    /// Get a plugin by name
    pub fn get(&self, name: &str) -> Option<Arc<dyn Plugin>> {
        self.plugins.read().ok()?.get(name).cloned()
    }

    /// List all enabled plugins
    pub fn list_enabled(&self) -> Vec<String> {
        self.enabled
            .read()
            .ok()
            .map(|e| e.clone())
            .unwrap_or_default()
    }

    /// Set enabled plugins from config
    pub fn set_enabled(&self, enabled_list: &[String]) -> Result<()> {
        let mut enabled = self
            .enabled
            .write()
            .map_err(|_| Error::Plugin("Lock poisoned".into()))?;
        *enabled = enabled_list.to_vec();
        Ok(())
    }

    /// Check if a plugin is enabled
    pub fn is_enabled(&self, name: &str) -> bool {
        self.enabled
            .read()
            .ok()
            .map(|e| e.contains(&name.to_string()))
            .unwrap_or(false)
    }
}
