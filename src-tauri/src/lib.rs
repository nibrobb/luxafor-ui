// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use luxafor::usb_hid::USBDeviceDiscovery;
use luxafor::Device;
use luxafor::SolidColor;

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
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![set_light_color])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
