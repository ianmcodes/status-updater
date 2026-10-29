mod config;
mod ha_client;
mod models;
mod tray;

use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::RwLock;

use models::{AppConfig, EntityState, EntitySummary};

pub struct AppState {
    pub config: Arc<RwLock<AppConfig>>,
    pub entity_state: Arc<RwLock<Option<EntityState>>>,
    pub http_client: reqwest::Client,
}

pub async fn refresh_state_internal(app: &AppHandle) -> Result<EntityState, String> {
    let state = app.state::<AppState>();
    let (ha_url, access_token, entity_id) = {
        let conf = state.config.read().await;
        (conf.ha_url.clone(), conf.access_token.clone(), conf.entity_id.clone())
    };

    if ha_url.trim().is_empty() || access_token.trim().is_empty() || entity_id.trim().is_empty() {
        let _ = tray::update_tray(app, None, None);
        return Err("Home Assistant configuration is incomplete. Please configure URL, Entity ID, and Token.".to_string());
    }

    match ha_client::fetch_entity(&state.http_client, &ha_url, &access_token, &entity_id).await {
        Ok(entity) => {
            {
                let mut current = state.entity_state.write().await;
                *current = Some(entity.clone());
            }
            let _ = tray::update_tray(app, Some(&entity), None);
            let _ = app.emit("ha-state-changed", &entity);
            Ok(entity)
        }
        Err(err) => {
            let _ = tray::update_tray(app, None, Some(&err));
            let _ = app.emit("ha-error", &err);
            Err(err)
        }
    }
}

pub async fn set_option_internal(app: &AppHandle, option: &str) -> Result<EntityState, String> {
    let state = app.state::<AppState>();
    let (ha_url, access_token, entity_id) = {
        let conf = state.config.read().await;
        (conf.ha_url.clone(), conf.access_token.clone(), conf.entity_id.clone())
    };

    if ha_url.trim().is_empty() || access_token.trim().is_empty() || entity_id.trim().is_empty() {
        return Err("Home Assistant configuration is incomplete.".to_string());
    }

    match ha_client::set_entity_option(
        &state.http_client,
        &ha_url,
        &access_token,
        &entity_id,
        option,
    )
    .await
    {
        Ok(entity) => {
            {
                let mut current = state.entity_state.write().await;
                *current = Some(entity.clone());
            }
            let _ = tray::update_tray(app, Some(&entity), None);
            let _ = app.emit("ha-state-changed", &entity);
            Ok(entity)
        }
        Err(err) => {
            let _ = tray::update_tray(app, None, Some(&err));
            let _ = app.emit("ha-error", &err);
            Err(err)
        }
    }
}

// ------------------- Tauri Commands -------------------

#[tauri::command]
async fn get_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    let conf = state.config.read().await;
    Ok(conf.clone())
}

#[tauri::command]
async fn save_config(
    app: AppHandle,
    state: State<'_, AppState>,
    new_config: AppConfig,
) -> Result<EntityState, String> {
    config::save_config(&app, &new_config)?;

    {
        let mut conf = state.config.write().await;
        *conf = new_config.clone();
    }

    // Refresh Home Assistant state with new configuration
    refresh_state_internal(&app).await
}

#[tauri::command]
async fn fetch_ha_state(app: AppHandle) -> Result<EntityState, String> {
    refresh_state_internal(&app).await
}

#[tauri::command]
async fn test_connection(
    state: State<'_, AppState>,
    ha_url: String,
    access_token: String,
    entity_id: String,
) -> Result<EntityState, String> {
    ha_client::fetch_entity(&state.http_client, &ha_url, &access_token, &entity_id).await
}

#[tauri::command]
async fn set_ha_option(app: AppHandle, option: String) -> Result<EntityState, String> {
    set_option_internal(&app, &option).await
}

#[tauri::command]
async fn discover_input_selects(
    state: State<'_, AppState>,
    ha_url: String,
    access_token: String,
) -> Result<Vec<EntitySummary>, String> {
    ha_client::discover_input_selects(&state.http_client, &ha_url, &access_token).await
}

#[tauri::command]
fn load_token_from_credentials() -> Result<String, String> {
    config::try_read_credentials_token()
        .ok_or_else(|| "No token found in credentials.json".to_string())
}

#[tauri::command]
fn hide_window(app: AppHandle) -> Result<(), String> {
    if let Some(w) = app.get_webview_window("main") {
        w.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

// ------------------- App Entrypoint -------------------

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // Keep app running in system tray when user closes the window
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .setup(|app| {
            let app_handle = app.handle().clone();

            // Load initial configuration
            let initial_config = config::load_config(&app_handle);
            let start_minimized = initial_config.start_minimized;

            let http_client = reqwest::Client::builder()
                .timeout(Duration::from_secs(12))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new());

            let app_state = AppState {
                config: Arc::new(RwLock::new(initial_config)),
                entity_state: Arc::new(RwLock::new(None)),
                http_client,
            };

            app.manage(app_state);

            // Create tray icon
            tray::create_tray(&app_handle).expect("failed to create system tray");

            // If start_minimized, hide the window
            if start_minimized {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.hide();
                }
            }

            // Initial state fetch in background
            let initial_app = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                let _ = refresh_state_internal(&initial_app).await;
            });

            // Background polling loop
            let poller_app = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                let mut last_poll = Instant::now();
                loop {
                    tokio::time::sleep(Duration::from_secs(3)).await;

                    let state = poller_app.state::<AppState>();
                    let interval_secs = {
                        let conf = state.config.read().await;
                        conf.refresh_interval_secs
                    };

                    if interval_secs > 0 && last_poll.elapsed() >= Duration::from_secs(interval_secs) {
                        last_poll = Instant::now();
                        let _ = refresh_state_internal(&poller_app).await;
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config,
            fetch_ha_state,
            test_connection,
            set_ha_option,
            discover_input_selects,
            load_token_from_credentials,
            hide_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
