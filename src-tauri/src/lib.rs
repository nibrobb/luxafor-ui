use std::str::FromStr;
use tauri::tray::{MouseButton, MouseButtonState, TrayIconEvent};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use luxafor::{usb_hid::USBDeviceDiscovery, Device, SolidColor};

use tauri::{
    menu::{AboutMetadataBuilder, Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    Manager, WindowEvent,
};

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(move |app| {
            let aboutmeta = AboutMetadataBuilder::new()
                .authors(Some(vec![String::from("Robin Kristiansen")]))
                .icon(Some(app.default_window_icon().unwrap().clone()))
                .build();
            let about_i = PredefinedMenuItem::about(app, Some("About"), Some(aboutmeta))?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let luxafor_ui_i = MenuItem::with_id(app, "luxafor-ui", "Luxafor-ui", true, None::<&str>)?;
            let menu = Menu::with_items(
                app,
                &[&luxafor_ui_i, &about_i, &PredefinedMenuItem::separator(app)?, &quit_i],
            )?;
            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("Luxafor-ui")
                .show_menu_on_left_click(true)
                .icon(app.default_window_icon().unwrap().clone())
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "luxafor-ui" => {
                        if let Some(window) = app.get_webview_window("main") {
                            window.show().unwrap();
                            window.unminimize().unwrap();
                            window.set_focus().unwrap();
                        }
                    },
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
                .build(app)?;
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
        .invoke_handler(tauri::generate_handler![set_light_color])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
