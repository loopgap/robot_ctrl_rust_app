#[cfg(windows)]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("../assets/branding/robot_control_app.ico");
    res.set("FileDescription", "Robot Control Suite");
    res.set("ProductName", "Robot Control Suite");
    res.set("OriginalFilename", "robot_control_rust.exe");
    if let Err(e) = res.compile() {
        eprintln!("Warning: Failed to compile windows resource: {}", e);
    }
}

#[cfg(not(windows))]
fn main() {}
