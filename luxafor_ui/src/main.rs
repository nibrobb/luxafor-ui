// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;
use luxafor_cmd_runner;

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let ui = MainUI::new()?;

    ui.on_button_blicky({
        // let ui_handle = ui.as_weak();
        move || {
            // let ui = ui_handle.unwrap();
            // let output = 
            luxafor_cmd_runner::open_notepad();
            // print!("{}", String::from_utf8_lossy(&output.stdout))
        }
    });

    ui.run()?;

    Ok(())
}
