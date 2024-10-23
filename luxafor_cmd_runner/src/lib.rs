use std::process::{Command, Output};


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
