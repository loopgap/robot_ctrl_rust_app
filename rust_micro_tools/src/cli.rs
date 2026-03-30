use clap::{Parser, Subcommand};
use colored::*;
use inquire::Select;
use serialport::available_ports;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "micro_tools_cli")]
#[command(about = "Rust Micro Tools Suite", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
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
                selected_baud.blue()
            );

            let pb = indicatif::ProgressBar::new_spinner();
            pb.enable_steady_tick(Duration::from_millis(120));
            pb.set_message(format!("Connecting to {} ...", selected_port));

            match serialport::new(&selected_port, selected_baud)
                .timeout(Duration::from_secs(3))
                .connect()
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
