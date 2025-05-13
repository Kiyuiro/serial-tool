use once_cell::sync::Lazy;
use serialport::{available_ports, SerialPortType};
use serialport::{SerialPort, SerialPortInfo};
use std::io::{Read, Write};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};
use serde::Serialize;
use tauri::{AppHandle, Emitter};

#[derive(Debug, serde::Deserialize)]
struct PortConfig {
    port: String,
    baud_rate: u32,
    data_bits: Option<u8>,
    stop_bits: Option<u8>,
    parity: Option<String>,
    timeout: Option<u64>,
}

#[derive(Serialize)]
struct PortName {
    name: String,
    port_type: String,
}

static GLOBAL_PORT: Lazy<Mutex<Option<Box<dyn SerialPort>>>> = Lazy::new(|| Mutex::new(None));
static LISTENER_RUNNING: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));

// 扫描串口
#[tauri::command]
fn scan_ports() -> String {
    let mut com_ports: Vec<PortName> = Vec::new();
    match available_ports() {
        Ok(ports) => {
            for p in ports {
                let name = p.port_name;
                let port_type = match p.port_type {
                    SerialPortType::UsbPort(info) => {
                        if let Some(product) = info.product {
                            product
                        } else if let Some(manufacturer) = info.manufacturer {
                            manufacturer
                        } else {
                            "USB".to_string()
                        }
                    }
                    SerialPortType::BluetoothPort => "蓝牙".to_string(),
                    SerialPortType::PciPort => "PCI".to_string(),
                    SerialPortType::Unknown => "未知".to_string(),
                };
                // com_ports.push(format!("{} ({})", port_name, port_type));
                com_ports.push(PortName {
                    name,
                    port_type
                });
            }
        }
        Err(e) => {
            eprintln!("Error listing serial ports: {}", e);
        }
    }
    serde_json::to_string(&com_ports).unwrap_or_else(|_| "[]".to_string())
}

// 打开串口
#[tauri::command]
async fn open_port(config: PortConfig) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let mut port = match serialport::new(&config.port, config.baud_rate)
            .timeout(Duration::from_millis(config.timeout.unwrap_or(1000)))
            .open()
        {
            Ok(p) => p,
            Err(e) => return Err(format!("打开串口失败: {}", e)),
        };
        port.flush().ok();
        let mut global = GLOBAL_PORT.lock().unwrap();
        *global = Some(port);
        println!("串口打开");
        Ok(())
    })
    .await
    .unwrap_or_else(|e| Err(format!("打开串口失败: {}", e)))
}

// 关闭串口
#[tauri::command]
async fn close_port() -> Result<(), String> {
    tokio::task::spawn_blocking(|| {
        let mut global = GLOBAL_PORT.lock().unwrap();
        if global.is_some() {
            *global = None; // 自动 drop 关闭串口
            println!("串口以关闭");
            Ok(())
        } else {
            Err("串口未打开".to_string())
        }
    })
    .await
    .unwrap_or_else(|e| Err(format!("关闭串口失败: {}", e)))
}

// 写入串口
#[tauri::command]
fn write_serial(data: Vec<u8>) -> Result<(), String> {
    let mut global = GLOBAL_PORT.lock().unwrap();
    if let Some(port) = global.as_mut() {
        match port.write_all(&data) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("写入失败: {}", e)),
        }
    } else {
        Err("串口未打开".to_string())
    }
}

// 开始监听串口
#[tauri::command]
fn start_serial_listener(app: AppHandle) -> Result<(), String> {
    let mut running = LISTENER_RUNNING.lock().unwrap();
    if *running {
        return Err("Listener already running.".to_string());
    }
    *running = true;
    drop(running); // 提前释放锁
    let app_handle = app.clone();
    thread::spawn(move || {
        let mut buffer = [0u8; 1024];

        loop {
            // 检查是否需要停止
            if !*LISTENER_RUNNING.lock().unwrap() {
                break;
            }

            // 复制出串口实例，避免长时间持有锁
            let maybe_data = {
                let mut port_lock = GLOBAL_PORT.lock().unwrap();
                if let Some(ref mut port) = *port_lock {
                    match port.read(&mut buffer) {
                        Ok(n) if n > 0 => Some(String::from_utf8_lossy(&buffer[..n]).to_string()),
                        Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => None,
                        Err(e) => {
                            eprintln!("Serial read error: {}", e);
                            // 通知前端串口出错
                            let _ = app_handle.emit("serial-error", e.to_string());
                            // 关闭串口
                            let _ = close_port();
                            // 关闭监听
                            let mut running = LISTENER_RUNNING.lock().unwrap();
                            *running = false;
                            // 退出
                            return;
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            };

            if let Some(data) = maybe_data {
                let _ = app_handle.emit("serial-data", data);
            }

            // 小睡以避免忙等
            thread::sleep(Duration::from_millis(1));
        }

        // 退出线程时清理
        let mut port = GLOBAL_PORT.lock().unwrap();
        *port = None;
        let mut running = LISTENER_RUNNING.lock().unwrap();
        *running = false;
    });
    Ok(())
}

// 停止监听串口
#[tauri::command]
fn stop_serial_listener() {
    let mut running = LISTENER_RUNNING.lock().unwrap();
    *running = false;
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            scan_ports,
            open_port,
            close_port,
            write_serial,
            start_serial_listener,
            stop_serial_listener,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
