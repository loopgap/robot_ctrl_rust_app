//! Library entry for `robot_control_rust`.
//!
//! The full app currently uses the binary target. This library surface keeps
//! workspace-level checks stable for tools that expect a valid lib target.

pub const ROBOT_CONTROL_CORE_VERSION: &str = env!("CARGO_PKG_VERSION");
