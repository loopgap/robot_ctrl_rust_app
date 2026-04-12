//! Robot Core - Shared Abstractions for Robot Control and Tools Suite
//!
//! This crate provides common traits and utilities used by both
//! robot_control and tools_suite applications.

pub mod config;
pub mod connection;
pub mod error;
pub mod plugin;

pub use config::PluginConfig;
pub use connection::ConnectionProvider;
pub use error::{Error, Result};
pub use plugin::{Plugin, PluginRegistry};
