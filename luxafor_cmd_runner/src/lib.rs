use std::process::{Command, Output};

pub fn busylight_on() -> Output {
    Command::new("busylight").args(["on"]).output().unwrap()
}

pub fn busylight_off() -> Output {
    Command::new("busylight").args(["off"]).output().unwrap()
}

pub enum BusylightColor {
    Red,
    Green,
    Blue,
}

pub fn busylight_set_solid(color: BusylightColor) -> Output {
    let out_color = match color {
        BusylightColor::Red => "red",
        BusylightColor::Green => "green",
        BusylightColor::Blue => "blue",
    };
    Command::new("busylight")
        .args(["on", out_color])
        .output()
        .unwrap()
}

// https://doc.rust-lang.org/std/process/struct.Command.html
pub fn open_notepad() -> Output {
    // let ... =  if cfg!(target_os = "windows") then {
    Command::new("cmd")
        .args(["/C", "notepad.exe"])
        .output()
        .expect("Something went wrong")
    // } else {
    //     ...
    // }
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
