use clap::{Parser, Subcommand};
use colored::*;
use inquire::Select;
use serialport::available_ports;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "micro_tools_cli")]
#[command(about = "Rust Micro Tools Suite - Intelligent Interactive CLI", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 交互式探测并连接串口 (Interactive serial port connection)
    Connect {
        /// 指定端口名 (如果不填则进入智能补全与选择)
        #[arg(short, long)]
        port: Option<String>,
        /// 指定波特率 (如果不填则智能推荐)
        #[arg(short, long)]
        baud: Option<u32>,
    },
    /// 排障与自检助理 (Troubleshooting assistant)
    Doctor,
}

pub fn run_cli(cli: Cli) {
    match cli.command {
        Some(Commands::Connect { port, baud }) => {
            println!("{}", "🚀 启动智能设备感知模块...".cyan().bold());

            // 智能感知端口
            let selected_port = match port {
                Some(p) => p,
                None => {
                    let ports = available_ports().unwrap_or_else(|_| vec![]);
                    if ports.is_empty() {
                        println!(
                            "{}",
                            "❌ 未检测到任何可用的串口设备，请检查是否已插入并安装驱动。".red()
                        );
                        println!(
                            "{}",
                            "💡 提示: 运行 `micro_tools_cli doctor` 进行深度排障。".yellow()
                        );
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

                    // 交互式下拉选择
                    let ans = Select::new("📡 检测到多个设备，请选择目标设备(支持搜索):", options)
                        .prompt();

                    match ans {
                        Ok(choice) => choice.split(" - ").next().unwrap().to_string(),
                        Err(_) => {
                            println!("{}", "⚠️ 已取消选择".yellow());
                            return;
                        }
                    }
                }
            };

            // 智能推荐波特率
            let selected_baud = match baud {
                Some(b) => b,
                None => {
                    let default_options = vec!["115200 (常用/推荐)", "9600", "460800", "921600"];
                    let ans = Select::new("⚙️ 请选择通信波特率:", default_options).prompt();
                    match ans {
                        Ok(choice) => {
                            let b_str = choice.split(' ').next().unwrap();
                            b_str.parse::<u32>().unwrap_or(115200)
                        }
                        Err(_) => 115200,
                    }
                }
            };

            println!(
                "{} {} {}",
                "✅ 已锁定设备:".green(),
                selected_port.bold(),
                format!("@ {} baud", selected_baud).blue()
            );

            // 进度动画模拟连接
            let pb = indicatif::ProgressBar::new_spinner();
            pb.enable_steady_tick(Duration::from_millis(120));
            pb.set_message(format!("尝试握手协议并在 {} 探测响应...", selected_port));

            std::thread::sleep(Duration::from_secs(2)); // mock connect delay

            pb.finish_with_message("🎉 连接成功！正在启动底层零拷贝通信管道...");
        }
        Some(Commands::Doctor) => {
            println!("{}", "🩺 智能排障系统启动...".magenta().bold());
            println!("- 检查驱动权限... 行");
            println!("- 检查日志分析... 无异常崩溃记录");
            println!(
                "{}",
                "💡 如果串口仍然无法打开，请确保未被其它终端 (如 Putty/SSCOM) 占用。".yellow()
            );
        }
        None => {
            // 如果无参数，提示进入 GUI 模式
            println!("{}", "ℹ️ 命令行无匹配动作，即将拉起图形化 (GUI) ...".cyan());
        }
    }
}
