use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use rfd::FileDialog;

pub fn open_text_file() -> Result<Option<(PathBuf, String)>, String> {
    let Some(path) = FileDialog::new()
        .add_filter(
            "Text",
            &[
                "txt", "log", "json", "csv", "jwt", "md", "pem", "key", "pub",
            ],
        )
        .pick_file()
    else {
        return Ok(None);
    };

    let mut result = Err(std::io::Error::other("Init"));
    let retries = if cfg!(target_os = "windows") { 3 } else { 1 };

    for _ in 0..retries {
        result = fs::read_to_string(&path);
        if result.is_ok() {
            break;
        }
        if cfg!(target_os = "windows") {
            thread::sleep(Duration::from_millis(300));
        }
    }

    let content = result.map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    Ok(Some((path, content)))
}

pub fn save_text_file(default_file_name: &str, content: &str) -> Result<Option<PathBuf>, String> {
    let Some(path) = FileDialog::new()
        .set_file_name(default_file_name)
        .save_file()
    else {
        return Ok(None);
    };

    let mut result = Err(std::io::Error::other("Init"));
    let retries = if cfg!(target_os = "windows") { 3 } else { 1 };

    for _ in 0..retries {
        result = fs::write(&path, content);
        if result.is_ok() {
            break;
        }
        if cfg!(target_os = "windows") {
            thread::sleep(Duration::from_millis(300));
        }
    }

    result.map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;
    Ok(Some(path))
}
