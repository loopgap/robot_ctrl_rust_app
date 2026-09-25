#[cfg(windows)]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("../assets/branding/rust_tools_suite_app.ico");
    res.set("FileDescription", "Rust Tools Suite");
    res.set("ProductName", "Rust Tools Suite");
    res.set("OriginalFilename", "rust_tools_suite.exe");
    if let Err(e) = res.compile() {
        eprintln!("Warning: Failed to compile windows resource: {}", e);
    }
}

#[cfg(not(windows))]
fn main() {}
