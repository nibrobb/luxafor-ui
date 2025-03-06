// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod slack_api;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // tracing_subscriber::fmt()
    //     .with_max_level(tracing::Level::DEBUG)
    //     .try_init()?;

    // let subscriber = tracing_subscriber::fmt().with_max_level(tracing::Level::DEBUG).init();  // shit printe dobbelt >:(

    let subscriber = tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(tracing::Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // span!(Level::TRACE, "luxafor_ui backend");

    luxafor_ui_lib::run()
}
