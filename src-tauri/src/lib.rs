use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{str::FromStr, time::Duration};
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconEvent},
    AppHandle, Runtime,
};
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use luxafor::{usb_hid::USBDeviceDiscovery, Device, SolidColor};
use tauri::{
    menu::{AboutMetadataBuilder, PredefinedMenuItem},
    tray::TrayIconBuilder,
    Manager, WindowEvent,
};

use tauri_plugin_store::StoreExt as _;
use tracing::*;

const PKG_NAME: &str = "Luxafor-ui";
const AUTHOR: &str = "Robin Kristiansen";
const COMMENTS: &str = "A simple app to control your Luxafor Flag";
const COPYRIGHT: &str = include_str!("copyright.txt");

#[cfg(any(feature = "slack_sync", feature = "slack_oauth"))]
pub mod slack_api;

#[cfg(feature = "slack_oauth")]
#[allow(unused_imports)]
use tauri_plugin_opener::open_url;

#[cfg(feature = "slack_sync")]
use slack_morphism::SlackUserProfile;

use slack_morphism::SlackApiTokenValue;

const STORE_FILENAME: &str = "store.json";

#[cfg(feature = "slack_sync")]
#[tracing::instrument]
fn color_to_profile(color: &SolidColor) -> SlackUserProfile {
    /* TODO: Use some centralized map e.g., the file "store.json", to store user-defined mappings
    between color and status, and vice versa */
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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SlackApiTokens {
    user_token: Option<SlackApiTokenValue>,
    bot_token: Option<SlackApiTokenValue>,
}

impl TryFrom<serde_json::Value> for SlackApiTokens {
    type Error = serde_json::Error;
    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        serde_json::from_value(value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppStore {
    slack_tokens: SlackApiTokens,
}

impl TryFrom<serde_json::Value> for AppStore {
    type Error = serde_json::Error;
    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        serde_json::from_value(value)
    }
}

impl Default for AppStore {
    fn default() -> Self {
        Self {
            slack_tokens: SlackApiTokens {
                user_token: None,
                bot_token: None,
            },
        }
    }
}

#[cfg(feature = "slack_sync")]
#[tracing::instrument]
async fn call_api(profile: SlackUserProfile, tokens: SlackApiTokens) -> Result<(), String> {
    let user_token = match tokens.user_token {
        Some(token) => token,
        None => return Err("user_token not found".to_string()),
    };
    match tauri::async_runtime::spawn(async move {
        use slack_morphism::SlackApiToken;
        slack_api::status_set(profile, SlackApiToken::new(user_token)).await
    })
    .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

fn append_action<R: Runtime>(
    actions_store: &tauri_plugin_store::Store<R>,
    new_item: serde_json::Value,
) -> tauri::Result<()> {
    // Get actions array
    let mut actions = actions_store
        .get("actions")
        .unwrap_or(serde_json::Value::Array(vec![]));

    // Check it's actually an array
    if let serde_json::Value::Array(ref mut arr) = actions {
        info!("Pushed {:?} into actions", new_item);
        arr.push(new_item);
    } else {
        // actions = serde_json::Value::Array(vec![new_item]);
        todo!("Figure out what to do here");
    }

    // Save the changes
    actions_store.set("actions", actions);
    actions_store.save().unwrap();
    Ok(())
}

#[tauri::command]
#[tracing::instrument(skip(app))]
async fn set_light_color(
    #[allow(unused_variables)] app: AppHandle,
    color: &str,
) -> Result<(), String> {
    span!(Level::DEBUG, "set_light_color");

    #[cfg(feature = "slack_sync")]
    let app_store = {
        let store_path = app
            .path()
            .app_config_dir()
            .map_err(|e| e.to_string())?
            .join(STORE_FILENAME);
        let store = app.store(store_path).map_err(|e| e.to_string())?;
        let app_store: SlackApiTokens = match store.get("slack_tokens") {
            Some(store) => match store.try_into() {
                Ok(store) => store,
                Err(e) => {
                    error!("Could not parse store: {}", e);
                    return Err(e.to_string());
                }
            },
            None => {
                error!("Could not get store");
                return Err("Could not get store".to_string());
            }
        };
        store.close_resource();
        app_store
    };

    let actions = app.store("actions.json").unwrap();

    let discovery = USBDeviceDiscovery::new().map_err(|e| e.to_string())?;
    let device = discovery.device().map_err(|e| e.to_string())?;
    debug!("set_light_color called");

    let s = color.to_lowercase();
    match s.as_str() {
        "off" => device.turn_off().map_err(|e| e.to_string()),
        _ => {
            if let Ok(parsed_color) = SolidColor::from_str(s.as_str()) {
                #[cfg(feature = "slack_sync")]
                {
                    let profile = color_to_profile(&parsed_color);
                    let tokens = app_store.clone();
                    call_api(profile, tokens).await?;
                }
                info!("Append foobar to actions");
                let _ = append_action(&actions, json!( {"foo": "bar"} ));

                device
                    .set_solid_color(parsed_color)
                    .map_err(|e| e.to_string())
            } else {
                Err(String::from("Invalid color"))
            }
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(move |app| {
            info!("Starting Luxafor-ui");

            // Create the config directory for Luxafor-ui if it does not exist
            let app_config_dir = app
                .path()
                .app_config_dir()
                .expect("Failed to resolve app config dir"); // Why would this ever fail?

            let store_path = app_config_dir.join(STORE_FILENAME);
            info!("Store path: {:?}", &store_path);

            // Create config directory and store file if they do not exist
            if !app_config_dir.exists() {
                std::fs::create_dir_all(&app_config_dir).expect("Failed to create app config dir");
                std::fs::File::create(&store_path).expect("Failed to create store file");
            }

            let handle = app.app_handle().clone();

            let _ = handle
                .store_builder("actions.json")
                .default("actions", serde_json::Value::Array(Vec::new()))
                .auto_save(Duration::from_secs(5))
                .build()?;

            // actions_store.save().unwrap();

            // Re-create the store file if the user deleted it
            if !store_path.exists() {
                std::fs::File::create(&store_path).expect("Failed to create store file");
            }
            let store = app.store(&store_path)?;

            // if store.is_empty() {
            //     todo!("Figure this shit out");
            // }

            #[cfg(feature = "slack_oauth")]
            tauri::async_runtime::spawn(async {
                slack_api::setup_oauth()
                    .await
                    .expect("Failed to setup oauth");
            });

            // let handle = app.handle();
            let aboutmeta = AboutMetadataBuilder::new()
                .name(Some(PKG_NAME))
                .authors(Some(vec![AUTHOR.into()]))
                .comments(Some(COMMENTS))
                .copyright(Some(COPYRIGHT))
                .icon(Some(handle.default_window_icon().unwrap().clone()))
                .build();

            let about_i = PredefinedMenuItem::about(&handle, Some("About"), Some(aboutmeta))?;

            let quit_i = MenuItemBuilder::with_id("quit", "Quit").build(&handle)?;

            let luxafor_ui_i = MenuItemBuilder::with_id("luxafor_ui", PKG_NAME).build(&handle)?;

            #[cfg(feature = "slack_oauth")]
            let add_to_slack_i =
                MenuItemBuilder::with_id("add_to_slack", "Add to Slack").build(handle)?;

            let menu = MenuBuilder::new(&handle)
                .items(&[
                    &luxafor_ui_i,
                    &about_i,
                    #[cfg(feature = "slack_oauth")]
                    &PredefinedMenuItem::separator(&handle)?,
                    #[cfg(feature = "slack_oauth")]
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
                    #[cfg(feature = "slack_oauth")]
                    "add_to_slack" => {
                        todo!("Implement logic to add to slack");
                        // debug!("Add to Slack pressed");
                        // open_url(slack_api::INSTALL_URL, None::<&str>).unwrap();
                        // let state = app.state::<Arc<Mutex<GlobalState>>>().lock().unwrap().clone();
                        // if let Some(ref token) = state.bot_token { debug!("BOT TOKEN:\t{}", token); }
                        // if let Some(ref token) = state.user_token { debug!("USER TOKEN:\t{}", token); }
                    }
                    #[cfg(feature = "slack_sync")]
                    "activate_slack_status_syncronization" => {
                        todo!("Implement logic to turn on/off Slack staus sync")
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
            // store.close_resource();
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
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![set_light_color,])
        .run(tauri::generate_context!())?;

    Ok(())
}
