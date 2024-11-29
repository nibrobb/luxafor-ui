// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use luxafor::{
    Device,
    SolidColor,
    usb_hid::USBDeviceDiscovery
};

use tauri::{
    menu::{Menu, MenuItem, AboutMetadataBuilder, PredefinedMenuItem},
    tray::TrayIconBuilder,
};

#[tauri::command]
fn set_light_color(color: &str) -> Result<(), String> {
    let discovery = USBDeviceDiscovery::new().map_err(|e| e.to_string())?;
    let device = discovery.device().map_err(|e| e.to_string())?;

    println!("USB Device: {:?}", device.id());

    let parsed_color = match color {
        "green" => SolidColor::Green,
        "red" => SolidColor::Red,
        "blue" => SolidColor::Blue,
        _ => return Err("Unsupported color".to_string()),
    };

    device
        .set_solid_color(parsed_color)
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let about_i = PredefinedMenuItem::about(
                app,
                Some("About"),
                Some(
                    AboutMetadataBuilder::new()
                        .authors(Some(vec![String::from("Robin Kristiansen")]))
                        .icon(Some(app.default_window_icon().unwrap().clone()))
                        .build(),
                ),
            )?;
            let menu = Menu::with_items(app, &[&about_i, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .menu_on_left_click(true)
                .icon(app.default_window_icon().unwrap().clone())
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        println!("Quit meny item was clicked");
                        app.exit(0);
                    }
                    _ => {
                        println!("Menu item {:?} not handled", event.id);
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
