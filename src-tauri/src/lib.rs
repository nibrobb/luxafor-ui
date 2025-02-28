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
use tracing::{debug, error};

#[tauri::command]
fn set_light_color(color: &str) -> Result<(), String> {
    let discovery = USBDeviceDiscovery::new().map_err(|e| e.to_string())?;
    let device = discovery.device().map_err(|e| e.to_string())?;

    let s = color.to_lowercase();
    let result = match s.as_str() {
        "off" => device.turn_off().map_err(|e| e.to_string()),
        _ => {
            if let Ok(parsed_color) = SolidColor::from_str(&s) {
                device
                    .set_solid_color(parsed_color)
                    .map_err(|e| e.to_string())
            } else {
                return Err(String::from("Invalid color"));
            }
        }
    };

    result
}

mod slack_api;

#[derive(Clone, Debug)]
pub struct Tokens {
    pub bot_token: Arc<Mutex<String>>,
    pub user_token: Arc<Mutex<String>>,
    pub counter: Arc<Mutex<i32>>,
}

#[tauri::command]
fn read_write_tokens(tokens_arc: tauri::State<Arc<Mutex<Tokens>>>) -> String {
    // simulate 10 threads
    for _ in 0..10 {
        let my_tokens_arc_clone = Arc::clone(&tokens_arc); // Clone arc to every thread, and share tokens_arc
        tauri::async_runtime::spawn(async move {
            let mut tokens = my_tokens_arc_clone.lock().unwrap();
            tokens.bot_token = Arc::new(Mutex::new("the BOT token goes here".into()));
            tokens.user_token = Arc::new(Mutex::new("the USER token goes here".into()));
            tokens.counter = Arc::new(Mutex::new(69));
        });
    }

    let tokens = tokens_arc.lock().unwrap();

    format!(
        "Tokens are:\nBOT:\t{}\nUSER:\t{}\n\nCounter:\t{}",
        tokens.bot_token.lock().unwrap(),
        tokens.user_token.lock().unwrap(),
        tokens.counter.lock().unwrap()
    )
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tauri::Builder::default()
        .manage(Tokens {
            bot_token: Arc::new(Mutex::new("BOT TOKEN".into())),
            user_token: Arc::new(Mutex::new("USER TOKEN".into())),
            counter: Arc::new(Mutex::new(0)),
        })
        .setup(move |app| {
            let my_tokens = Arc::new(Mutex::new(
                Tokens {
                    bot_token: Arc::new(Mutex::new("BOT TOKEN".into())),
                    user_token: Arc::new(Mutex::new("USER TOKEN".into())),
                    counter: Arc::new(Mutex::new(0)),
                }
            ));
            app.manage(my_tokens);

            tauri::async_runtime::spawn(async move {
                match slack_api::setup_oauth().await {
                    Ok(()) => (),
                    Err(e) => {
                        error!("{}", e);
                        ()
                    }
                }
            });

            let handle = app.handle();

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

            let add_to_slack_i =
                MenuItemBuilder::with_id("add_to_slack", "Add to Slack").build(handle)?;

            let menu = MenuBuilder::new(handle)
                .items(&[
                    &luxafor_ui_i,
                    &about_i,
                    &PredefinedMenuItem::separator(handle)?,
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
                    "add_to_slack" => {
                        // Open a web browser and navigate to http://localhost:8080/auth/install
                        debug!("Add to Slack pressed: {}", read_write_tokens(app.state()));
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| match event {
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
            Ok(())
        })
        .on_window_event(|window, event| {
            if let Some(main_window) = window.app_handle().get_webview_window("main") {
                match event {
                    WindowEvent::CloseRequested { api, .. } => {
                        api.prevent_close();
                        main_window.hide().unwrap();
                    }
                    _ => {}
                }
            }
        })
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            set_light_color,
            read_write_tokens,
        ])
        .run(tauri::generate_context!())?;

    Ok(())
}
