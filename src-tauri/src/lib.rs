use std::{
    str::FromStr,
    sync::{Arc, Mutex},
};
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconEvent},
};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use luxafor::{usb_hid::USBDeviceDiscovery, Device, SolidColor};
use tauri::{
    menu::{AboutMetadataBuilder, PredefinedMenuItem},
    tray::TrayIconBuilder,
    Manager, WindowEvent,
};

use tauri_plugin_store::StoreExt;
use tracing::debug;

#[cfg(feature = "slack_oauth")]
use tauri_plugin_opener::open_url;

mod slack_api;

#[tauri::command]
async fn call_api(color: &str) -> Result<(), String> {
    // But not here...
    let color = color.to_string();
    tokio::task::spawn_local(async move {
        slack_api::send_status(color)
            .await
            .map_err(|e| e.to_string())
    });
    // slack_api::send_status(color.to_string()).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn set_light_color(color: &str) -> Result<(), String> {
    let discovery = USBDeviceDiscovery::new().map_err(|e| e.to_string())?;
    let device = discovery.device().map_err(|e| e.to_string())?;

    let s = color.to_lowercase();
    match s.as_str() {
        "off" => device.turn_off().map_err(|e| e.to_string()),
        _ => {
            if let Ok(parsed_color) = SolidColor::from_str(&s) {
                device
                    .set_solid_color(parsed_color)
                    .map_err(|e| e.to_string())
            } else {
                Err(String::from("Invalid color"))
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Tokens {
    pub bot_token: Option<String>,
    pub user_token: Option<String>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .setup(move |app| {
            debug!("Setting up application");

            // Shit works here...
            tauri::async_runtime::spawn(async { slack_api::send_status("yellow".to_string()).await.expect("What the fuck"); });

            app.manage(Arc::new(Mutex::new(Tokens {
                bot_token: Some("BOT TOKEN".into()),
                user_token: Some("USER TOKEN".into()),
            })));

            // `Tokens` is already managed, so `manage()` returns false
            assert!(!app.manage(Arc::new(Mutex::new(Tokens {
                bot_token: Some("BOT TOKEN".into()),
                user_token: Some("USER TOKEN".into()),
            }))));
            let store = app.store("store.json")?;
            if let Some(bot_token) = store.get("bot_token") {
                if let Some(user_token) = store.get("user_token") {
                    app.state::<Arc<Mutex<Tokens>>>().lock().unwrap().bot_token = Some(bot_token.to_string());
                    app.state::<Arc<Mutex<Tokens>>>().lock().unwrap().user_token = Some(user_token.to_string());
                }
            }

            let handle = app.handle();
            #[cfg(feature = "slack_oauth")]
            tauri::async_runtime::spawn(async {
                slack_api::setup_oauth().await.expect("Failed to setup oauth");
            });

            let aboutmeta = AboutMetadataBuilder::new()
                .name(Some("Luxafor-ui"))
                .authors(Some(vec![String::from("Robin Kristiansen")]))
                .comments(Some("A simple app to control your Luxafor Flag"))
                .copyright(Some("Luxafor-ui is not affiliated with, endorsed by, or associated with Luxafor. Luxafor is a registered trademark of GreyNut SIA."))
                .icon(Some(handle.default_window_icon().unwrap().clone()))
                .build();

            let about_i = PredefinedMenuItem::about(handle, Some("About"), Some(aboutmeta))?;

            let quit_i = MenuItemBuilder::with_id("quit", "Quit").build(handle)?;

            let luxafor_ui_i =
                MenuItemBuilder::with_id("luxafor_ui", "Luxafor-ui").build(handle)?;

            #[cfg(feature = "slack_oauth")]
            let add_to_slack_i =
                MenuItemBuilder::with_id("add_to_slack", "Add to Slack").build(handle)?;

            let menu = MenuBuilder::new(handle)
                .items(&[
                    &luxafor_ui_i,
                    &about_i,
                    #[cfg(feature = "slack_oauth")]
                    &PredefinedMenuItem::separator(handle)?,
                    #[cfg(feature = "slack_oauth")]
                    &add_to_slack_i,
                    &PredefinedMenuItem::separator(handle)?,
                    &quit_i,
                ])
                .build()?;
            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("Luxafor-ui")
                .show_menu_on_left_click(true)
                .icon(handle.default_window_icon().unwrap().clone())
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "luxafor_ui" => {
                        if let Some(window) = app.get_webview_window("main") {
                            window.show().unwrap();
                            window.unminimize().unwrap();
                            window.set_focus().unwrap();
                        }
                    }
                    #[cfg(feature = "slack_oauth")]
                    "add_to_slack" => {
                        debug!("Add to Slack pressed");

                        open_url(slack_api::INSTALL_URL, None::<&str>).unwrap();
                        let state = app.state::<Arc<Mutex<Tokens>>>().lock().unwrap().clone();
                        if let Some(ref token) = state.bot_token { debug!("BOT TOKEN:\t{}", token); }
                        if let Some(ref token) = state.user_token { debug!("USER TOKEN:\t{}", token); }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| match event {
                    TrayIconEvent::DoubleClick {
                        button: MouseButton::Left,
                        ..
                    } |
                    TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } => {
                        if let Some(window) = tray.app_handle().get_webview_window("main") {
                            window.show().unwrap();
                            window.unminimize().unwrap();
                            window.set_focus().unwrap();
                        }
                    }
                    _ => {}
                })
                .build(handle)?;

            store.close_resource();

            Ok(())
        })
        .on_window_event(|window, event| {
            if let Some(main_window) = window.app_handle().get_webview_window("main") {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    main_window.hide().unwrap();
                }
            }
        })
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            set_light_color,
            call_api,
        ])
        .run(tauri::generate_context!())?;

    Ok(())
}
