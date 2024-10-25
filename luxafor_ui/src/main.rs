// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;
use luxafor_cmd_runner;
use luxafor_cmd_runner::BusylightColor;

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let ui = MainUI::new()?;

    ui.on_button_green({
        move || {
            luxafor_cmd_runner::busylight_set_solid(BusylightColor::Green);
        }
    });
    ui.on_button_red({
        move || {
            luxafor_cmd_runner::busylight_set_solid(BusylightColor::Red);
        }
    });
    ui.on_button_off({
        move || {
            luxafor_cmd_runner::busylight_off();
        }
    });

    ui.run()?;

    Ok(())
}
