use crate::at32_boot_entry::{
    enter_bootloader, list_serial_endpoints, load_key_file, SerialEndpoint,
};
use crate::i18n::Language;
use crate::theme::ACCENT_COLOR;
use eframe::egui;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

/// Interactive AT32 application-authenticated Bootloader entry.
/// The selected key path is session-only and is deliberately not exposed to
/// the preferences model.
pub struct At32BootEntryTool {
    port: String,
    baud: String,
    endpoints: Vec<SerialEndpoint>,
    key_path: String,
    status: String,
    result: Option<Receiver<Result<(), String>>>,
}

impl Default for At32BootEntryTool {
    fn default() -> Self {
        let endpoints = list_serial_endpoints();
        let port = endpoints
            .first()
            .map(|endpoint| endpoint.name.clone())
            .unwrap_or_else(|| "COM1".into());
        Self {
            port,
            baud: "115200".into(),
            endpoints,
            key_path: String::new(),
            status: String::new(),
            result: None,
        }
    }
}

impl At32BootEntryTool {
    pub fn clear(&mut self) {
        *self = Self::default();
    }
    pub fn output_text(&self) -> Option<&str> {
        (!self.status.is_empty()).then_some(&self.status)
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context, lang: Language) {
        if let Some(result) = &self.result {
            match result.try_recv() {
                Ok(Ok(())) => {
                    self.status = lang.tr("认证成功；设备将复位至 Bootloader。请重新连接并查询 STATUS 后再刷写。", "Authenticated; the device is resetting to Bootloader. Reconnect and query STATUS before flashing.");
                    self.result = None;
                }
                Ok(Err(error)) => {
                    self.status = format!("{}: {error}", lang.tr("失败", "Failed"));
                    self.result = None;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.status = lang.tr(
                        "后台操作意外结束",
                        "Background operation ended unexpectedly",
                    );
                    self.result = None;
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }

        ui.heading(lang.tr("AT32 Bootloader 入口", "AT32 Bootloader Entry"));
        ui.label(lang.tr("流程：1 选择 USB 串口 · 2 应用认证 · 3 设备以 Bootloader 身份重新枚举。产品密钥只在本次操作读取，不保存到偏好或设备档案。", "Flow: 1 select USB serial · 2 authenticate the application · 3 the device re-enumerates as Bootloader. The product key is read only for this operation and is never saved."));
        ui.add_space(12.0);
        ui.group(|ui| {
            ui.strong(lang.tr("1. 选择连接", "1. Select connection"));
            ui.label(lang.tr("兼容 USB CDC、CH340、CP210x、FTDI、蓝牙 SPP、虚拟 COM 与原生串口。设备元数据仅用于识别，不会限制连接。", "Supports USB CDC, CH340, CP210x, FTDI, Bluetooth SPP, virtual COM and native serial. Metadata helps identification only; it never blocks a connection."));
            ui.horizontal_wrapped(|ui| {
                egui::ComboBox::from_label(lang.tr("检测到的端口", "Detected ports"))
                    .width(360.0)
                    .selected_text(self.endpoints.iter().find(|item| item.name == self.port).map(|item| format!("{} — {}", item.name, item.description)).unwrap_or_else(|| self.port.clone()))
                    .show_ui(ui, |ui| {
                        for endpoint in &self.endpoints { ui.selectable_value(&mut self.port, endpoint.name.clone(), format!("{} — {}", endpoint.name, endpoint.description)); }
                    });
                if ui.add_sized([92.0, 32.0], egui::Button::new(lang.tr("刷新端口", "Refresh ports"))).clicked() {
                    self.endpoints = list_serial_endpoints();
                    if !self.endpoints.iter().any(|endpoint| endpoint.name == self.port) { self.status = lang.tr("已刷新端口列表；可手动输入端口名以连接未报告元数据的设备。", "Port list refreshed; enter a port name manually for devices without reported metadata."); }
                }
            });
            ui.horizontal(|ui| {
                ui.label(lang.tr("手动端口", "Manual port"));
                ui.add_sized([150.0, 28.0], egui::TextEdit::singleline(&mut self.port).hint_text("COM5 / /dev/ttyACM0"));
                egui::ComboBox::from_label("Baud").selected_text(&self.baud).show_ui(ui, |ui| {
                    for rate in ["9600", "57600", "115200", "230400", "460800", "921600"] { ui.selectable_value(&mut self.baud, rate.to_string(), rate); }
                });
            });
        });
        ui.add_space(10.0);
        ui.group(|ui| {
            ui.strong(lang.tr("2. 认证", "2. Authenticate"));
            ui.horizontal(|ui| {
                ui.label(lang.tr("Boot-entry 密钥文件", "Boot-entry key file"));
                ui.add_sized(
                    [ui.available_width() - 105.0, 24.0],
                    egui::TextEdit::singleline(&mut self.key_path).hint_text("32-byte hex key"),
                );
                if ui.button(lang.tr("选择…", "Choose…")).clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .set_title("Boot-entry key (not stored)")
                        .pick_file()
                    {
                        self.key_path = path.display().to_string();
                    }
                }
            });
        });
        ui.add_space(8.0);
        let running = self.result.is_some();
        if ui
            .add_enabled(
                !running && !self.port.trim().is_empty() && !self.key_path.trim().is_empty(),
                egui::Button::new(
                    lang.tr("认证并进入 Bootloader", "Authenticate and Enter Bootloader"),
                )
                .fill(ACCENT_COLOR)
                .min_size(egui::vec2(260.0, 36.0)),
            )
            .clicked()
        {
            let port = self.port.trim().to_string();
            let key_path = PathBuf::from(self.key_path.trim());
            let baud = self.baud.trim().parse::<u32>().unwrap_or(115200);
            let (sender, receiver) = mpsc::channel();
            self.result = Some(receiver);
            self.status = lang.tr("正在认证…", "Authenticating…");
            thread::spawn(move || {
                let outcome = (|| {
                    let mut key = load_key_file(&key_path).map_err(|e| e.to_string())?;
                    let nonce = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map(|d| d.subsec_nanos())
                        .unwrap_or(1)
                        .max(1);
                    let mut device = serialport::new(&port, baud)
                        .timeout(std::time::Duration::from_millis(80))
                        .open()
                        .map_err(|e| e.to_string())?;
                    let result =
                        enter_bootloader(&mut *device, &key, nonce).map_err(|e| e.to_string());
                    key.fill(0);
                    result
                })();
                let _ = sender.send(outcome);
            });
        }
        if running {
            ui.spinner();
            ui.label(lang.tr(
                "正在与设备通信；请勿拔出 USB 或关闭窗口。",
                "Communicating with the device; do not unplug USB or close this window.",
            ));
        }
        if !self.status.is_empty() {
            ui.add_space(8.0);
            ui.label(&self.status);
        }
    }
}
