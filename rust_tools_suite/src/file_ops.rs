use std::fs;
use std::path::PathBuf;

use rfd::FileDialog;

pub fn open_text_file() -> Result<Option<(PathBuf, String)>, String> {
    let Some(path) = FileDialog::new()
        .add_filter(
            "Text",
            &["txt", "log", "json", "csv", "jwt", "md", "pem", "key"],
        )
        .pick_file()
    else {
        return Ok(None);
    };

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    Ok(Some((path, content)))
}

pub fn save_text_file(default_file_name: &str, content: &str) -> Result<Option<PathBuf>, String> {
    let Some(path) = FileDialog::new()
        .set_file_name(default_file_name)
        .save_file()
    else {
        return Ok(None);
    };

    fs::write(&path, content).map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;
    Ok(Some(path))
}
