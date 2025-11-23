use std::{str::FromStr, time::Duration};

use luxafor::{usb_hid::USBDeviceDiscovery, Device, SolidColor};
use tauri::{
    menu::{AboutMetadataBuilder, MenuBuilder, MenuItemBuilder, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, WindowEvent,
};

#[allow(unused_imports)]
use tauri_plugin_store::StoreExt;

const PKG_NAME: &str = "Luxafor-ui";
const AUTHOR: &str = "Robin Kristiansen";
const COMMENTS: &str = "A simple app to control your Luxafor Flag";
const COPYRIGHT: &str = include_str!("copyright.txt");
const SETTINGS_FILENAME: &str = "settings.json";

#[cfg(feature = "slack_sync")]
const SESSION_URL: &str = env!("SLACK_SESSION_URL");
#[cfg(feature = "slack_sync")]
const SESSION_STATUS_URL: &str = env!("SLACK_SESSION_STATUS_URL");

#[cfg(feature = "slack_sync")]
mod slack_api;

#[cfg(feature = "slack_sync")]
use {slack_api::SlackApiTokens, slack_morphism::SlackUserProfile};

#[cfg(feature = "slack_sync")]
fn color_to_profile(color: SolidColor) -> SlackUserProfile {
    // TODO: Get mappings from `settings.json`
    match color {
        SolidColor::Red => SlackUserProfile::new()
            .with_status_text("Opptatt".into())
            .with_status_emoji(":no_entry:".into()),
        SolidColor::Green => SlackUserProfile::new()
            .with_status_text("".into())
            .with_status_emoji("".into()),
        SolidColor::Blue => SlackUserProfile::new()
            .with_status_text("I\'m blue, baby!".into())
            .with_status_emoji(":blueberries:".into()),
        SolidColor::Cyan => SlackUserProfile::new()
            .with_status_text("".into())
            .with_status_emoji(":raccoon:".into()),
        // TODO: Add all colors
        _ => SlackUserProfile::new()
            .with_status_text("".into())
            .with_status_emoji("".into()),
    }
}

// TODO: Implement this
// fn profile_to_color(profile: &SlackUserProfile) -> SolidColor { ... }

#[cfg_attr(feature = "tracing", tracing::instrument(skip(app)))]
#[tauri::command]
async fn set_light_color(
    #[allow(unused_variables)] app: AppHandle,
    color: &str,
) -> Result<(), String> {
    #[cfg(feature = "slack_sync")]
    let tokens: SlackApiTokens = {
        let store_path = app
            .path()
            .app_config_dir()
            .map_err(|e| e.to_string())?
            .join(SETTINGS_FILENAME);
        let store = app
            .get_store(store_path)
            .ok_or("Could not get store".to_string())?;
        // TODO: Abstract to helper function
        let slack_tokens: SlackApiTokens = match store.get("slack_tokens") {
            Some(value) => match value.try_into() {
                Ok(tokens) => tokens,
                Err(e) => {
                    #[cfg(feature = "tracing")]
                    tracing::error!("Could not parse `slack_tokens`: {}", e);
                    return Err(e.to_string());
                }
            },
            None => {
                return Err("Key `slack_tokens` not found in `settings.json`".to_string());
            }
        };
        slack_tokens
    };

    let discovery = USBDeviceDiscovery::new().map_err(|e| e.to_string())?;
    let device = discovery.device().map_err(|e| e.to_string())?;
    #[cfg(feature = "tracing")]
    tracing::debug!("Found device: {}", device.id());

    let s = color.to_lowercase();
    match s.as_str() {
        "off" => device.turn_off().map_err(|e| e.to_string()),
        _ => {
            if let Ok(parsed_color) = SolidColor::from_str(s.as_str()) {
                let res = device
                    .set_solid_color(parsed_color.clone())
                    .map_err(|e| e.to_string());
                #[cfg(feature = "slack_sync")]
                {
                    let profile = color_to_profile(parsed_color);
                    let tokens = tokens.clone();
                    slack_api::slack_set_profile(profile, tokens).await?;
                }
                res
            } else {
                Err(String::from("Invalid color"))
            }
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let builder = tauri::Builder::default()
        .setup(move |app| {
            #[cfg(feature = "tracing")]
            tracing::info!("Starting Luxafor-ui");
            #[cfg(all(feature = "slack_sync", feature = "tracing"))]
            {
                tracing::debug!("SESSION_URL: {}", SESSION_URL);
                tracing::debug!("SESSION_STATUS_URL: {}", SESSION_STATUS_URL);
            }

            let app_config_dir = app
                .path()
                .app_config_dir()
                .expect("Failed to resolve app config dir");

            let settings_path = app_config_dir.join(SETTINGS_FILENAME);
            #[cfg(feature = "tracing")]
            tracing::info!("Settings path: {:?}", settings_path);

            let handle = app.app_handle().clone();

            #[allow(unused_mut)]
            let mut settings_json_default = std::collections::HashMap::new();
            #[cfg(feature = "slack_sync")]
            settings_json_default.insert(
                "slack_tokens".to_string(),
                serde_json::to_value(SlackApiTokens::default())?,
            );
            // TODO: Insert color/profile-mappings

            #[allow(unused)]
            let store = tauri_plugin_store::StoreBuilder::new(app, settings_path)
                .auto_save(Duration::from_secs(5))
                .defaults(settings_json_default)
                .build()?;

            // if store.is_empty() {
            //     store.set(
            //         "slack_tokens",
            //         serde_json::to_value(SlackApiTokens::default())?,
            //     )
            // }

            #[cfg(feature = "tracing")]
            tracing::debug!("Store contents:\n{:#?}", store.entries());

            let about_meta = AboutMetadataBuilder::new()
                .name(Some(PKG_NAME))
                .authors(Some(vec![AUTHOR.into()]))
                .comments(Some(COMMENTS))
                .copyright(Some(COPYRIGHT))
                .icon(Some(handle.default_window_icon().unwrap().clone()))
                .build();

            let about_i = PredefinedMenuItem::about(&handle, Some("About"), Some(about_meta))?;

            let quit_i = MenuItemBuilder::with_id("quit", "Quit").build(&handle)?;

            let luxafor_ui_i = MenuItemBuilder::with_id("luxafor_ui", PKG_NAME).build(&handle)?;

            #[cfg(feature = "slack_sync")]
            let add_to_slack_i =
                MenuItemBuilder::with_id("add_to_slack", "Add to Slack").build(&handle)?;

            let menu = MenuBuilder::new(&handle)
                .items(&[
                    &luxafor_ui_i,
                    &about_i,
                    #[cfg(feature = "slack_sync")]
                    &PredefinedMenuItem::separator(&handle)?,
                    #[cfg(feature = "slack_sync")]
                    &add_to_slack_i,
                    &PredefinedMenuItem::separator(&handle)?,
                    &quit_i,
                ])
                .build()?;

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .tooltip(PKG_NAME)
                .show_menu_on_left_click(true)
                .icon(handle.default_window_icon().unwrap().clone())
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "luxafor_ui" => {
                        if let Some(window) = app.get_webview_window("main") {
                            window.show().unwrap();
                            window.unminimize().unwrap();
                            window.set_focus().unwrap();
                        }
                    }
                    #[cfg(feature = "slack_sync")]
                    "add_to_slack" => {
                        #[cfg(feature = "tracing")]
                        tracing::debug!("Add to Slack pressed");
                        // TODO: Sent POST request to ``SESSION_URL``
                        //  Parse response and grab `authorization URL`
                        //  Open `authorization URL` in the browser
                        //  While waiting for the OAuth flow to complete, poll the `SESSION_STATUS_URL`
                        //  Until it returns a 200 OK with the tokens
                        //  Then store them in the store (`settings.json`)
                        let authorization_url = "https://nibrobb.dev"; // Placeholder
                        tauri_plugin_opener::open_url(authorization_url, None::<&str>).unwrap();
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| match event {
                    TrayIconEvent::DoubleClick {
                        button: MouseButton::Left,
                        ..
                    }
                    | TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } => {
                        if let Some(window) = tray.app_handle().get_webview_window("main") {
                            window.show().unwrap();
                            window.unminimize().unwrap();
                            window.set_focus().unwrap();
                        }
                    }
                    _ => {}
                })
                .build(&handle)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let Some(main_window) = window.app_handle().get_webview_window("main") {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    main_window.hide().unwrap();
                }
            }
        })
        .plugin(tauri_plugin_store::Builder::default().build());

    #[cfg(feature = "slack_sync")]
    let builder = builder.plugin(tauri_plugin_opener::init());

    builder
        .invoke_handler(tauri::generate_handler![set_light_color,])
        .run(tauri::generate_context!())?;

    Ok(())
}
