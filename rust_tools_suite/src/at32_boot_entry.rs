//! AT32M416 application-side Boot-entry authentication protocol.
//!
//! This module deliberately owns only the wire-independent authentication and
//! UART text exchange.  The key is loaded for a single operation and is never
//! part of the suite preferences or device profile.

use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

type HmacSha256 = Hmac<Sha256>;

const DOMAIN: &[u8] = b"AT32M416-BOOT-ENTRY-V1\0";
const PHASE_SIGNATURE: u32 = 0x3154_4841;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BeginReply {
    pub key_id: u8,
    pub next_counter: u32,
    pub scope_id: [u8; 16],
}

/// A cross-platform serial endpoint. `serialport` normalises USB CDC ACM,
/// FTDI/CP210x/CH34x bridges, Bluetooth SPP and native UARTs behind this
/// single list; hardware metadata is advisory rather than a compatibility
/// gate because many bridges expose no stable USB identifiers.
#[cfg(feature = "gui")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SerialEndpoint {
    pub name: String,
    pub description: String,
}

#[cfg(feature = "gui")]
pub fn list_serial_endpoints() -> Vec<SerialEndpoint> {
    let mut endpoints: Vec<SerialEndpoint> = serialport::available_ports()
        .unwrap_or_default()
        .into_iter()
        .map(|port| {
            let description = match port.port_type {
                serialport::SerialPortType::UsbPort(info) => {
                    let maker = info
                        .manufacturer
                        .unwrap_or_else(|| "USB serial bridge".into());
                    let product = info.product.unwrap_or_default();
                    let serial = info
                        .serial_number
                        .map(|value| format!(" · {value}"))
                        .unwrap_or_default();
                    format!(
                        "USB {maker} {product} · VID:{:04X} PID:{:04X}{serial}",
                        info.vid, info.pid
                    )
                }
                serialport::SerialPortType::BluetoothPort => "Bluetooth serial (SPP)".into(),
                serialport::SerialPortType::PciPort => "Native / PCI serial".into(),
                serialport::SerialPortType::Unknown => {
                    "Serial endpoint (metadata unavailable)".into()
                }
            };
            SerialEndpoint {
                name: port.port_name,
                description,
            }
        })
        .collect();
    endpoints.sort_by(|left, right| left.name.cmp(&right.name));
    endpoints
}

#[derive(Debug, thiserror::Error)]
pub enum BootEntryError {
    #[error("boot key must contain exactly 32 hexadecimal bytes")]
    InvalidKey,
    #[error("device returned an invalid BOOT_AUTH_BEGIN reply: {0}")]
    InvalidBeginReply(String),
    #[error("device rejected boot-entry request: {0}")]
    Rejected(String),
    #[error("serial I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("timed out waiting for device reply")]
    Timeout,
}

pub fn load_key_file(path: &Path) -> Result<[u8; 32], BootEntryError> {
    let text = fs::read_to_string(path).map_err(BootEntryError::Io)?;
    let hex: String = text.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    if hex.len() != 64 {
        return Err(BootEntryError::InvalidKey);
    }
    let mut key = [0u8; 32];
    for (idx, byte) in key.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&hex[idx * 2..idx * 2 + 2], 16)
            .map_err(|_| BootEntryError::InvalidKey)?;
    }
    Ok(key)
}

pub fn make_tag(
    key: &[u8; 32],
    key_id: u8,
    scope_id: &[u8; 16],
    next_counter: u32,
    nonce: u32,
) -> [u8; 16] {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC accepts all key lengths");
    mac.update(DOMAIN);
    mac.update(&[1u8, key_id, 0u8, 0u8]); // UART source, key ID, reserved
    mac.update(&next_counter.to_le_bytes());
    mac.update(&nonce.to_le_bytes());
    mac.update(&PHASE_SIGNATURE.to_le_bytes());
    mac.update(scope_id);
    let digest = mac.finalize().into_bytes();
    let mut tag = [0u8; 16];
    tag.copy_from_slice(&digest[..16]);
    tag
}

pub fn parse_begin_reply(reply: &str) -> Result<BeginReply, BootEntryError> {
    if !reply.trim_start().starts_with("OK") {
        return Err(BootEntryError::Rejected(reply.trim().into()));
    }
    let mut kid = None;
    let mut ctr = None;
    let mut scope = None;
    for item in reply.split_ascii_whitespace() {
        if let Some(value) = item.strip_prefix("KID=") {
            kid = value.parse::<u8>().ok();
        }
        if let Some(value) = item.strip_prefix("CTR=") {
            ctr = value.parse::<u32>().ok();
        }
        if let Some(value) = item.strip_prefix("SCOPE=") {
            let value: String = value.chars().filter(|c| c.is_ascii_hexdigit()).collect();
            if value.len() == 32 {
                let mut bytes = [0u8; 16];
                for (i, b) in bytes.iter_mut().enumerate() {
                    *b = u8::from_str_radix(&value[i * 2..i * 2 + 2], 16).unwrap_or(0);
                }
                scope = Some(bytes);
            }
        }
    }
    match (kid, ctr, scope) {
        (Some(key_id), Some(next_counter), Some(scope_id)) => Ok(BeginReply {
            key_id,
            next_counter,
            scope_id,
        }),
        _ => Err(BootEntryError::InvalidBeginReply(reply.trim().into())),
    }
}

fn read_line(
    port: &mut dyn serialport::SerialPort,
    timeout: Duration,
) -> Result<String, BootEntryError> {
    let started = Instant::now();
    let mut data = Vec::new();
    let mut byte = [0u8; 1];
    while started.elapsed() < timeout {
        match std::io::Read::read(port, &mut byte) {
            Ok(1) => {
                data.push(byte[0]);
                if byte[0] == b'\n' {
                    return Ok(String::from_utf8_lossy(&data).into_owned());
                }
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::TimedOut => {}
            Err(error) => return Err(BootEntryError::Io(error)),
        }
    }
    Err(BootEntryError::Timeout)
}

fn request(port: &mut dyn serialport::SerialPort, text: &str) -> Result<String, BootEntryError> {
    std::io::Write::write_all(port, text.as_bytes())?;
    std::io::Write::flush(port)?;
    read_line(port, Duration::from_millis(900))
}

pub fn enter_bootloader(
    port: &mut dyn serialport::SerialPort,
    key: &[u8; 32],
    nonce: u32,
) -> Result<(), BootEntryError> {
    let begin = request(port, &format!("#CMD:BOOT_AUTH_BEGIN={nonce:08X}\r\n"))?;
    let reply = parse_begin_reply(&begin)?;
    let tag = make_tag(
        key,
        reply.key_id,
        &reply.scope_id,
        reply.next_counter,
        nonce,
    );
    let tag_text: String = tag.iter().map(|b| format!("{b:02X}")).collect();
    let tag_reply = request(port, &format!("#CMD:BOOT_AUTH_TAG={tag_text}\r\n"))?;
    if !tag_reply.trim_start().starts_with("OK") {
        return Err(BootEntryError::Rejected(tag_reply.trim().into()));
    }
    let commit = request(port, "#CMD:BOOT_AUTH_COMMIT\r\n")?;
    if !commit.trim_start().starts_with("OK") {
        return Err(BootEntryError::Rejected(commit.trim().into()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn begin_reply_is_strict() {
        assert_eq!(
            parse_begin_reply("OK KID=3 MODE=PRODUCT CTR=9 SCOPE=000102030405060708090A0B0C0D0E0F")
                .unwrap()
                .next_counter,
            9
        );
        assert!(parse_begin_reply("OK CTR=9").is_err());
    }
    #[test]
    fn tag_changes_with_counter() {
        let key = [7u8; 32];
        let scope = [9u8; 16];
        assert_ne!(
            make_tag(&key, 1, &scope, 1, 2),
            make_tag(&key, 1, &scope, 2, 2)
        );
    }
}
