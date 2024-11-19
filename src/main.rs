// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use luxafor::usb_hid::{USBDevice, USBDeviceDiscovery};
use luxafor::{Device, SolidColor, SpecificLED, TargetedDevice};

use std::error::Error;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tao::platform::unix::EventLoopBuilderExtUnix;

use tray_icon::{
    menu::{AboutMetadata, Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    TrayIconBuilder, TrayIconEvent,
};
slint::include_modules!();

fn discover_device() -> luxafor::error::Result<USBDevice> {
    let discovery = USBDeviceDiscovery::new()?;
    let device = discovery.device()?;
    println!("USB device: '{}'", device.id());
    Ok(device)
}

fn set_do_not_disturb() -> luxafor::error::Result<()> {
    let mut device = discover_device()?;
    device.set_specific_led(SpecificLED::AllFront)?;
    device.set_solid_color(SolidColor::Magenta)
}

fn set_free() -> luxafor::error::Result<()> {
    let mut device = discover_device()?;
    device.set_specific_led(SpecificLED::AllFront)?;
    device.set_solid_color(SolidColor::Green)
}

fn set_off() -> luxafor::error::Result<()> {
    let device = discover_device()?;
    // device.set_specific_led(SpecificLED::All)?;
    device.turn_off()
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

fn load_muda_icon(path: &std::path::Path) -> tray_icon::menu::Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open(path)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    tray_icon::menu::Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}

fn main() -> Result<(), Box<dyn Error>> {
    let (tx, rx) = mpsc::channel();

    // Spawn a new thread for the `tao` tray icon shit
    let tray_icon_thread = thread::spawn(move || {
        let icon_path = concat!(env!("CARGO_MANIFEST_DIR"), "/images/tauri_icon.png");
        let event_loop = EventLoopBuilder::new().with_any_thread(true).build();
        let tray_menu = Menu::new();
        let luxafor_ui_i = MenuItem::new("Luxafor UI", true, None);
        let quit_i = MenuItem::new("Quit", true, None);

        let muda_icon = load_muda_icon(std::path::Path::new(icon_path));
        tray_menu
            .append_items(&[
                &luxafor_ui_i,
                &PredefinedMenuItem::about(
                    None,
                    Some(AboutMetadata {
                        name: Some("Luxafor UI".to_string()),
                        copyright: Some("Copyright Robin Kristiansen © 2024".to_string()),
                        authors: Some(vec!["Robin Kristiansen".to_string()]),
                        license: Some("<INSERT LICENSE>".to_string()),
                        icon: Some(muda_icon),
                        ..Default::default()
                    }),
                ),
                &PredefinedMenuItem::separator(),
                &quit_i,
            ])
            .unwrap();

        let mut tray_icon = None;

        let menu_channel = MenuEvent::receiver();
        let tray_channel = TrayIconEvent::receiver();

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
                        .unwrap(),
                );
            }
            if let Ok(event) = menu_channel.try_recv() {
                if event.id == quit_i.id() {
                    tray_icon.take();
                    tx.send("quit").unwrap();
                    *control_flow = ControlFlow::Exit;
                } else if event.id == luxafor_ui_i.id() {
                    tx.send("show").unwrap();
                }
                println!("{event:?}");
            }
            if let Ok(event) = tray_channel.try_recv() {
                println!("{event:?}");
            }
        });
    });

    // Main thread message loop
    let main_ui_thread = thread::spawn(move || {
        let main_ui = Arc::new(Mutex::new(MainUI::new().unwrap()));
        {
            let main_ui = main_ui.clone();
            main_ui.lock().unwrap().on_button_off({
                || {
                    if set_off().is_ok() {
                        println!("Set off");
                    }
                }
            });
            main_ui.lock().unwrap().on_button_green({
                || {
                    if set_free().is_ok() {
                        println!("Set free")
                    }
                }
            });
            main_ui.lock().unwrap().on_button_red({
                || {
                    if set_do_not_disturb().is_ok() {
                        println!("Set do not disturb")
                    }
                }
            });
        }
        let main_ui_clone = main_ui.clone();
        main_ui.lock().unwrap().show().unwrap();
        slint::run_event_loop_until_quit().unwrap();
        slint::spawn_local(async move {
            loop {
                if let Ok(msg) = rx.try_recv() {
                    let main_ui = main_ui_clone.lock().unwrap();
                    match msg {
                        "quit" => {
                            main_ui.hide().unwrap();
                            println!("Received msg `quit`");
                            break;
                        }
                        "show" => {
                            main_ui.show().unwrap();
                            println!("Received msg `show`");
                        }
                        _ => {}
                    }
                }
            }
        }).unwrap();
    });

    tray_icon_thread.join().unwrap();
    main_ui_thread.join().unwrap();

    Ok(())
}