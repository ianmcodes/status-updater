use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use crate::models::AppConfig;

pub fn get_config_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("Failed to resolve app config dir: {}", e))?;
    Ok(dir.join("config.json"))
}

pub fn try_read_credentials_token() -> Option<String> {
    // Check current working directory or relative path
    let candidates = [
        PathBuf::from("credentials.json"),
        PathBuf::from("../credentials.json"),
    ];

    for path in &candidates {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(token) = json.get("ha-access-token").and_then(|v| v.as_str()) {
                    let trimmed = token.trim();
                    if !trimmed.is_empty() {
                        return Some(trimmed.to_string());
                    }
                }
            }
        }
    }
    None
}

pub fn load_config(app: &AppHandle) -> AppConfig {
    let path = match get_config_path(app) {
        Ok(p) => p,
        Err(_) => return AppConfig::default(),
    };

    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(mut config) = serde_json::from_str::<AppConfig>(&content) {
                // If token is empty in config, check if credentials.json has one
                if config.access_token.trim().is_empty() {
                    if let Some(token) = try_read_credentials_token() {
                        config.access_token = token;
                    }
                }
                return config;
            }
        }
    }

    let mut config = AppConfig::default();
    if let Some(token) = try_read_credentials_token() {
        config.access_token = token;
    }
    config
}

pub fn save_config(app: &AppHandle, config: &AppConfig) -> Result<(), String> {
    let path = get_config_path(app)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    let json = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize configuration: {}", e))?;

    std::fs::write(&path, json)
        .map_err(|e| format!("Failed to write configuration file: {}", e))?;

    Ok(())
}
