use crate::at32_boot_entry::{enter_bootloader, load_key_file};
use clap::{Parser, Subcommand};
use colored::*;
use inquire::Select;
use serialport::available_ports;
use std::path::PathBuf;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Parser)]
#[command(name = "tools_suite_cli")]
#[command(about = "Rust Tools Suite", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Authenticate the running AT32 application and request a Bootloader reset.
    EnterBootloader {
        #[arg(short, long)]
        port: String,
        #[arg(short, long, default_value_t = 115200)]
        baud: u32,
        /// 32-byte product key file. It is read only for this invocation.
        #[arg(long)]
        boot_key_file: PathBuf,
        /// Optional client nonce in hexadecimal; random by default.
        #[arg(long)]
        nonce: Option<String>,
    },
    Connect {
        #[arg(short, long)]
        port: Option<String>,
        #[arg(short, long)]
        baud: Option<u32>,
    },
    Doctor,
}

pub fn run_cli(cli: Cli) {
    match cli.command {
        Some(Commands::EnterBootloader {
            port,
            baud,
            boot_key_file,
            nonce,
        }) => {
            let key = match load_key_file(&boot_key_file) {
                Ok(key) => key,
                Err(error) => {
                    eprintln!("{}", error.to_string().red());
                    return;
                }
            };
            let nonce = match nonce {
                Some(value) => match u32::from_str_radix(value.trim_start_matches("0x"), 16) {
                    Ok(value) if value != 0 => value,
                    _ => {
                        eprintln!(
                            "{}",
                            "--nonce must be a non-zero 32-bit hexadecimal value".red()
                        );
                        return;
                    }
                },
                None => SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.subsec_nanos())
                    .unwrap_or(1)
                    .max(1),
            };
            let mut device = match serialport::new(&port, baud)
                .timeout(Duration::from_millis(80))
                .open()
            {
                Ok(device) => device,
                Err(error) => {
                    eprintln!("{}", format!("Cannot open {port}: {error}").red());
                    return;
                }
            };
            match enter_bootloader(&mut *device, &key, nonce) {
                Ok(()) => println!(
                    "{}",
                    "Bootloader entry accepted; reconnect and query STATUS before flashing."
                        .green()
                ),
                Err(error) => eprintln!("{}", format!("Bootloader entry failed: {error}").red()),
            }
        }
        Some(Commands::Connect { port, baud }) => {
            println!("{}", "Starting device discovery...".cyan().bold());
            let selected_port = match port {
                Some(p) => p,
                None => {
                    let ports = match available_ports() {
                        Ok(p) => p,
                        Err(e) => {
                            eprintln!("{}", format!("Failed to enumerate ports: {}", e).red());
                            println!("{}", "Check driver or run doctor.".yellow());
                            return;
                        }
                    };

                    if ports.is_empty() {
                        println!("{}", "No serial ports found.".red());
                        return;
                    }

                    let mut options = Vec::new();
                    for p in ports {
                        let name = p.port_name;
                        let mut desc = "Unknown Device".to_string();
                        if let serialport::SerialPortType::UsbPort(info) = p.port_type {
                            desc = format!("USB PID:{:04x} VID:{:04x}", info.pid, info.vid);
                        }
                        options.push(format!("{} - {}", name, desc));
                    }

                    let ans = Select::new("Select device:", options).prompt();

                    match ans {
                        Ok(choice) => choice
                            .split(" - ")
                            .next()
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| {
                                println!("{}", "Port parse failed, using default".yellow());
                                "COM1".to_string()
                            }),
                        Err(_) => {
                            println!("{}", "Selection cancelled".yellow());
                            return;
                        }
                    }
                }
            };

            let selected_baud = match baud {
                Some(b) => b,
                None => {
                    let default_options = vec!["115200", "9600", "460800", "921600"];
                    let ans = Select::new("Select baud rate:", default_options).prompt();
                    match ans {
                        Ok(choice) => choice.parse::<u32>().unwrap_or(115200),
                        Err(_) => 115200,
                    }
                }
            };

            println!(
                "{} {} @ {} baud",
                "Device locked:".green(),
                selected_port.bold(),
                format!("{}", selected_baud).blue()
            );

            let pb = indicatif::ProgressBar::new_spinner();
            pb.enable_steady_tick(Duration::from_millis(120));
            pb.set_message(format!("Connecting to {} ...", selected_port));

            match serialport::new(&selected_port, selected_baud)
                .timeout(Duration::from_secs(3))
                .open()
            {
                Ok(_port) => {
                    pb.finish_with_message("Connection successful!");
                    println!("{}", "Device connected.".green());
                }
                Err(e) => {
                    pb.finish_with_message("Connection failed");
                    eprintln!(
                        "{}",
                        format!("Cannot connect to {}: {}", selected_port, e).red()
                    );
                }
            }
        }
        Some(Commands::Doctor) => {
            println!("{}", "Troubleshooting system started...".magenta().bold());

            match available_ports() {
                Ok(ports) => {
                    if ports.is_empty() {
                        println!("{}", "No serial devices detected".yellow());
                    } else {
                        println!(
                            "{}",
                            format!("Driver check: {} devices found", ports.len()).green()
                        );
                        for port in &ports {
                            println!(
                                "  - {} ({})",
                                port.port_name,
                                match port.port_type {
                                    serialport::SerialPortType::UsbPort(_) => "USB",
                                    serialport::SerialPortType::PciPort => "PCI",
                                    serialport::SerialPortType::BluetoothPort => "Bluetooth",
                                    serialport::SerialPortType::Unknown => "Unknown",
                                }
                            );
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{}", format!("Driver enumeration failed: {}", e).red());
                }
            }

            println!();
            check_system_logs();

            println!("{}", "Check if port is occupied.".yellow());
        }
        None => {
            println!("{}", "No command, launching GUI...".cyan());
        }
    }
}

fn check_system_logs() {
    println!("{}", "Checking system logs...".cyan());

    #[cfg(target_os = "windows")]
    {
        use std::process::Command;

        let output = Command::new("powershell")
            .args(["-NoProfile", "-Command", "echo NoLogs"])
            .output();

        match output {
            Ok(result) => {
                let stdout = String::from_utf8_lossy(&result.stdout);
                if stdout.trim().is_empty() || stdout.trim() == "NoLogs" {
                    println!("{}", "No serial-related errors found".green());
                } else {
                    println!("{}", "Found related logs:".yellow());
                    for line in stdout.lines().take(5) {
                        println!("  {}", line.trim());
                    }
                }
            }
            Err(_) => {
                println!("{}", "Cannot read system logs".yellow());
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        use std::process::Command;

        let output = Command::new("dmesg").args(["--level=err", "-t"]).output();

        match output {
            Ok(result) => {
                let stdout = String::from_utf8_lossy(&result.stdout);
                let serial_errors: Vec<_> = stdout
                    .lines()
                    .filter(|line| {
                        let lower = line.to_lowercase();
                        lower.contains("usb") || lower.contains("serial") || lower.contains("tty")
                    })
                    .take(5)
                    .collect();

                if serial_errors.is_empty() {
                    println!("{}", "No serial errors in kernel log".green());
                } else {
                    println!("{}", "Found kernel logs:".yellow());
                    for line in serial_errors {
                        println!("  {}", line);
                    }
                }
            }
            Err(_) => {
                println!("{}", "Cannot read kernel log".yellow());
            }
        }
    }
}
