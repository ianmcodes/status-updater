use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager};

pub const TRAY_ID: &str = "ha-status-tray";

pub fn update_tray(
    app: &AppHandle,
    entity: Option<&crate::models::EntityState>,
    err_msg: Option<&str>,
) -> tauri::Result<()> {
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return Ok(());
    };

    let menu = Menu::new(app)?;

    match (entity, err_msg) {
        (Some(e), _) => {
            let label = e.friendly_name.as_deref().unwrap_or(&e.entity_id);
            let header_title = format!("● {}: {}", label, e.state);
            let header = MenuItem::with_id(app, "header_info", &header_title, false, None::<&str>)?;
            menu.append(&header)?;
            menu.append(&PredefinedMenuItem::separator(app)?)?;

            for opt in &e.options {
                let is_checked = opt == &e.state;
                let item_id = format!("ha_opt:{}", opt);
                let item = CheckMenuItem::with_id(app, &item_id, opt, true, is_checked, None::<&str>)?;
                menu.append(&item)?;
            }

            menu.append(&PredefinedMenuItem::separator(app)?)?;
            let refresh = MenuItem::with_id(app, "refresh_now", "🔄 Refresh", true, None::<&str>)?;
            menu.append(&refresh)?;
            let settings = MenuItem::with_id(app, "open_settings", "⚙️ Settings...", true, None::<&str>)?;
            menu.append(&settings)?;
            menu.append(&PredefinedMenuItem::separator(app)?)?;
            let quit = MenuItem::with_id(app, "quit_app", "❌ Exit", true, None::<&str>)?;
            menu.append(&quit)?;

            let tooltip = format!("{}: {}", label, e.state);
            let _ = tray.set_tooltip(Some(tooltip));
        }
        (None, Some(err)) => {
            let header_title = format!("⚠️ Error: {}", err);
            let header = MenuItem::with_id(app, "header_info", &header_title, false, None::<&str>)?;
            menu.append(&header)?;
            menu.append(&PredefinedMenuItem::separator(app)?)?;
            let retry = MenuItem::with_id(app, "refresh_now", "🔄 Retry Connection", true, None::<&str>)?;
            menu.append(&retry)?;
            let settings = MenuItem::with_id(app, "open_settings", "⚙️ Settings...", true, None::<&str>)?;
            menu.append(&settings)?;
            menu.append(&PredefinedMenuItem::separator(app)?)?;
            let quit = MenuItem::with_id(app, "quit_app", "❌ Exit", true, None::<&str>)?;
            menu.append(&quit)?;

            let _ = tray.set_tooltip(Some("HA Status: Error".to_string()));
        }
        (None, None) => {
            let header = MenuItem::with_id(app, "header_info", "⚠️ Not Configured", false, None::<&str>)?;
            menu.append(&header)?;
            menu.append(&PredefinedMenuItem::separator(app)?)?;
            let settings = MenuItem::with_id(app, "open_settings", "⚙️ Open Settings...", true, None::<&str>)?;
            menu.append(&settings)?;
            menu.append(&PredefinedMenuItem::separator(app)?)?;
            let quit = MenuItem::with_id(app, "quit_app", "❌ Exit", true, None::<&str>)?;
            menu.append(&quit)?;

            let _ = tray.set_tooltip(Some("HA Status Updater (Not Configured)".to_string()));
        }
    }

    tray.set_menu(Some(menu))?;
    Ok(())
}

pub fn create_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let initial_menu = Menu::new(app)?;
    let header = MenuItem::with_id(app, "header_info", "● Loading HA Status...", false, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let settings = MenuItem::with_id(app, "open_settings", "⚙️ Settings...", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit_app", "❌ Exit", true, None::<&str>)?;

    initial_menu.append(&header)?;
    initial_menu.append(&sep)?;
    initial_menu.append(&settings)?;
    initial_menu.append(&sep)?;
    initial_menu.append(&quit)?;

    let icon = app.default_window_icon().cloned().expect("missing default icon");

    let _tray = TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .menu(&initial_menu)
        .show_menu_on_left_click(true)
        .tooltip("Home Assistant Status Updater")
        .on_menu_event(|app, event| {
            let id = event.id().as_ref();
            match id {
                "quit_app" => {
                    app.exit(0);
                }
                "open_settings" => {
                    if let Some(w) = app.get_webview_window("main") {
                        let _ = w.show();
                        let _ = w.unminimize();
                        let _ = w.set_focus();
                    }
                }
                "refresh_now" => {
                    let app_clone = app.clone();
                    tauri::async_runtime::spawn(async move {
                        let _ = crate::refresh_state_internal(&app_clone).await;
                    });
                }
                opt if opt.starts_with("ha_opt:") => {
                    let option_name = opt.trim_start_matches("ha_opt:").to_string();
                    let app_clone = app.clone();
                    tauri::async_runtime::spawn(async move {
                        let _ = crate::set_option_internal(&app_clone, &option_name).await;
                    });
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::DoubleClick {
                button: MouseButton::Left,
                ..
            } = event
            {
                if let Some(w) = tray.app_handle().get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.unminimize();
                    let _ = w.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
}
