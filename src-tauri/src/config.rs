use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

// 配置文件
#[derive(Serialize, Deserialize, Debug)]
pub struct AppConfig {
    pub baud_rate: Option<u32>, // 波特率
    pub send_data: Option<String>, // 发送数据
    pub hex_send: Option<bool>, // 十六进制发送
    pub hex_display: Option<bool>, // 十六进制显示
    pub auto_scroll: Option<bool>, // 自动滚动
    pub save_send_msg: Option<bool>, // 是否保留发送区
    pub received_text_color: Option<String>, // 接收字符串颜色
    pub send_text_color: Option<String>, // 发送字符串颜色
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            baud_rate: Some(9600),
            send_data: Some("".to_string()),
            hex_send: Some(false),
            hex_display: Some(false),
            auto_scroll: Some(true),
            save_send_msg: Some(false),
            received_text_color: Some("#000000".to_string()),
            send_text_color: Some("#FFAA00".to_string()),
        }
    }
}

pub fn save_config(config: &AppConfig) -> Result<(), String> {
    let config_path = get_config_file_path().ok_or("无法获取配置路径")?;
    let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::create_dir_all(config_path.parent().unwrap()).map_err(|e| e.to_string())?;
    fs::write(config_path, json).map_err(|e| e.to_string())
}

pub fn load_config() -> AppConfig {
    let path = get_config_file_path();
    if let Some(config_path) = path {
        if config_path.exists() {
            if let Ok(data) = fs::read_to_string(config_path) {
                if let Ok(config) = serde_json::from_str::<AppConfig>(&data) {
                    return config;
                }
            }
        }
    }
    AppConfig::default()
}

fn get_config_file_path() -> Option<PathBuf> {
    let mut path = dirs::config_dir()?;
    path.push(".serial-tool");
    path.push("config.json");
    Some(path)
}