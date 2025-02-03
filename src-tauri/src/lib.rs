use std::str::FromStr;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use luxafor::{usb_hid::USBDeviceDiscovery, Device, SolidColor};

use tauri::{
    menu::{AboutMetadataBuilder, Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
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
        .setup(|app| {
            let aboutmeta = AboutMetadataBuilder::new()
                .authors(Some(vec![String::from("Robin Kristiansen")]))
                .icon(Some(app.default_window_icon().unwrap().clone()))
                .build();
            let about_i = PredefinedMenuItem::about(app, Some("About"), Some(aboutmeta))?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(
                app,
                &[&about_i, &PredefinedMenuItem::separator(app)?, &quit_i],
            )?;
            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(true)
                .icon(app.default_window_icon().unwrap().clone())
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {
                        eprintln!("Menu item {:?} not handled", event.id);
                    }
                })
                .build(app)?;
            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![set_light_color])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
