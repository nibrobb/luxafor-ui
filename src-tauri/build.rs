fn main() {
    println!(
        "cargo::rustc-env=SLACK_SESSION_URL={}",
        env!("SLACK_SESSION_URL")
    );
    tauri_build::build()
}
