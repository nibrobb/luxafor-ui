fn main() {
    println!(
        "cargo::rustc-env=SLACK_OAUTH_URL={}",
        env!("SLACK_OAUTH_URL")
    );
    tauri_build::build()
}
