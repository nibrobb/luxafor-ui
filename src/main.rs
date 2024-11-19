// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use luxafor::usb_hid::USBDeviceDiscovery;
use luxafor::{Device, SolidColor, SpecificLED, TargetedDevice};
use std::error::Error;
use std::thread;

slint::include_modules!();

use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tao::platform::unix::EventLoopBuilderExtUnix;
use tray_icon::{
    menu::{AboutMetadata, Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    TrayIconBuilder, TrayIconEvent,
};

fn set_do_not_disturb() -> luxafor::error::Result<()> {
    let discovery = USBDeviceDiscovery::new()?;
    let mut device = discovery.device()?;
    println!("USB device: '{}'", device.id());
    device.set_specific_led(SpecificLED::AllFront).expect("TODO: panic message");
    Ok(device.set_solid_color(SolidColor::Red).expect("TODO: panic message"))
}

fn set_free() -> luxafor::error::Result<()> {
    let discovery = USBDeviceDiscovery::new()?;
    let mut device = discovery.device()?;
    println!("USB device: '{}'", device.id());
    device.set_specific_led(SpecificLED::All).expect("TODO: panic message");
    Ok(device.set_solid_color(SolidColor::Green).expect("TODO: panic message"))
}

fn load_icon(path: &std::path::Path) -> tray_icon::Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open(path)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    tray_icon::Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}

fn main() -> Result<(), Box<dyn Error>> {
    let ui = MainUI::new()?;

    ui.on_button_off({
        let ui_handle = ui.as_weak();
        move || {
            // Hide window
            ui_handle.upgrade().unwrap().hide().expect("Hide failed");
        }
    });
    ui.on_button_green({
        move || {
            match set_free() {
                Ok(_) => println!("Set free"),
                Err(e) => println!("Error: {}", e),
            }
        }
    });
    ui.on_button_red({
        move || {
            match set_do_not_disturb() {
                Ok(_) => println!("Set do not disturb"),
                Err(e) => println!("Error: {}", e),
            }
        }
    });

    // Spawn a new thread for the `tao` tray icon shit
    let tao_thread = thread::spawn(move || {
        let icon_path = concat!(env!("CARGO_MANIFEST_DIR"), "/images/tauri_icon.png");
        let event_loop = EventLoopBuilder::new().with_any_thread(true).build();
        let tray_menu = Menu::new();
        let luxafor_ui_i = MenuItem::new("Luxafor UI", true, None);
        let quit_i = MenuItem::new("Quit", true, None);

        tray_menu.append_items(&[
            &luxafor_ui_i,
            &PredefinedMenuItem::about(
                None,
                Some(AboutMetadata {
                    name: Some("Luxafor UI".to_string()),
                    copyright: Some("Copyright Robin Kristiansen (c) 2024".to_string()),
                    authors: Some(vec!["Robin Kristiansen".to_string()]),
                    license: Some("<INSERT LICENSE>".to_string()),
                    ..Default::default()
                }),
            ),
            &PredefinedMenuItem::separator(),
            &quit_i,
        ]).unwrap();

        let mut tray_icon = None;

        let menu_channel = MenuEvent::receiver();
        let tray_channel = TrayIconEvent::receiver();

        // let tray_icon_thread: std::thread::JoinHandle<_> = std::thread::spawn(move || -> () {
        // Set 16ms delay (ish 60 fps)
        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::WaitUntil(
                std::time::Instant::now() + std::time::Duration::from_millis(16),
            );
            if let tao::event::Event::NewEvents(tao::event::StartCause::Init) = event {
                let icon = load_icon(std::path::Path::new(icon_path));
                tray_icon = Some(
                    TrayIconBuilder::new()
                        .with_menu(Box::new(tray_menu.clone()))
                        .with_tooltip("Luxafor UI")
                        .with_icon(icon)
                        .build()
                        .unwrap()
                );
            }
            if let Ok(event) = menu_channel.try_recv() {
                if event.id == quit_i.id() {
                    tray_icon.take();
                    *control_flow = ControlFlow::Exit;
                } else {
                    println!("{event:?}");
                }
            }
            if let Ok(event) = tray_channel.try_recv() {
                println!("{event:?}");
            }
        });
    });

    ui.run()?;
    slint::run_event_loop_until_quit()?;
    tao_thread.join().unwrap();

    Ok(())
}
