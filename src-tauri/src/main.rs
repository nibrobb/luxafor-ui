// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use tracing::{span, Level};
use tracing_subscriber::fmt::Subscriber;

mod slack_api;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let subscriber = Subscriber::builder().with_max_level(Level::DEBUG).finish();
    tracing::subscriber::set_global_default(subscriber).expect("Setting default subscriber failed");

    span!(Level::TRACE, "luxafor_ui backend");

    luxafor_ui_lib::run()
}
