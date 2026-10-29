use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};
use crate::models::AppConfig;

pub fn get_config_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("Failed to resolve app config dir: {}", e))?;
    Ok(dir.join("config.json"))
}

pub fn parse_credentials_token(content: &str) -> Option<String> {
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(content) {
        if let Some(token) = json.get("ha-access-token").and_then(|v| v.as_str()) {
            let trimmed = token.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

pub fn try_read_credentials_token() -> Option<String> {
    let candidates = [
        PathBuf::from("credentials.json"),
        PathBuf::from("../credentials.json"),
    ];

    for path in &candidates {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Some(token) = parse_credentials_token(&content) {
                return Some(token);
            }
        }
    }
    None
}

pub fn load_config_from_path(path: &Path) -> Option<AppConfig> {
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(config) = serde_json::from_str::<AppConfig>(&content) {
                return Some(config);
            }
        }
    }
    None
}

pub fn save_config_to_path(path: &Path, config: &AppConfig) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    let json = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize configuration: {}", e))?;

    std::fs::write(path, json)
        .map_err(|e| format!("Failed to write configuration file: {}", e))?;

    Ok(())
}

pub fn load_config(app: &AppHandle) -> AppConfig {
    let path = match get_config_path(app) {
        Ok(p) => p,
        Err(_) => return AppConfig::default(),
    };

    if let Some(mut config) = load_config_from_path(&path) {
        if config.access_token.trim().is_empty() {
            if let Some(token) = try_read_credentials_token() {
                config.access_token = token;
            }
        }
        return config;
    }

    let mut config = AppConfig::default();
    if let Some(token) = try_read_credentials_token() {
        config.access_token = token;
    }
    config
}

pub fn save_config(app: &AppHandle, config: &AppConfig) -> Result<(), String> {
    let path = get_config_path(app)?;
    save_config_to_path(&path, config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_credentials_token_valid() {
        let json = r#"{"ha-access-token": "eyJhbGciOiJIUzI1NiJ9.test.sig"}"#;
        assert_eq!(
            parse_credentials_token(json).as_deref(),
            Some("eyJhbGciOiJIUzI1NiJ9.test.sig")
        );
    }

    #[test]
    fn test_parse_credentials_token_empty_or_whitespace() {
        assert_eq!(parse_credentials_token(r#"{"ha-access-token": ""}"#), None);
        assert_eq!(parse_credentials_token(r#"{"ha-access-token": "   "}"#), None);
        assert_eq!(parse_credentials_token(r#"{"other-key": "token"}"#), None);
        assert_eq!(parse_credentials_token("not valid json"), None);
    }

    #[test]
    fn test_save_and_load_config_file() {
        let temp_dir = std::env::temp_dir().join(format!("ha_test_config_{}", std::process::id()));
        let config_file = temp_dir.join("config.json");

        let sample_config = AppConfig {
            ha_url: "http://test-ha.local:8123".into(),
            entity_id: "input_select.test_entity".into(),
            access_token: "test_token_abc".into(),
            refresh_interval_secs: 15,
            start_minimized: true,
        };

        // Save
        let save_res = save_config_to_path(&config_file, &sample_config);
        assert!(save_res.is_ok());

        // Load
        let loaded = load_config_from_path(&config_file);
        assert_eq!(loaded, Some(sample_config));

        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_load_nonexistent_or_corrupted_config() {
        let temp_dir = std::env::temp_dir().join(format!("ha_test_corrupt_{}", std::process::id()));
        let nonexistent = temp_dir.join("none.json");
        assert_eq!(load_config_from_path(&nonexistent), None);

        let corrupt = temp_dir.join("corrupt.json");
        let _ = std::fs::create_dir_all(&temp_dir);
        let _ = std::fs::write(&corrupt, "this is not json");
        assert_eq!(load_config_from_path(&corrupt), None);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
