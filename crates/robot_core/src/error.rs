//! Error handling for robot_core

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Plugin error: {0}")]
    Plugin(String),

    #[error("Config error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

pub type Result<T> = std::result::Result<T, Error>;
