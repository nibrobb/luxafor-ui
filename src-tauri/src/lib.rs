use std::str::FromStr;

use luxafor::{usb_hid::USBDeviceDiscovery, Device, SolidColor};
use tauri::{
    menu::{AboutMetadataBuilder, MenuBuilder, MenuItemBuilder, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, WindowEvent,
};

#[cfg(feature = "slack_sync")]
use tauri_plugin_deep_link::DeepLinkExt;
#[allow(unused_imports)]
use tauri_plugin_store::StoreExt;

const PKG_NAME: &str = "Luxafor-ui";
const AUTHOR: &str = "Robin Kristiansen";
const COMMENTS: &str = "A simple app to control your Luxafor Flag";
const COPYRIGHT: &str = include_str!("copyright.txt");
const SETTINGS_FILENAME: &str = "settings.json";

#[cfg(feature = "slack_sync")]
mod slack_api;

#[cfg(feature = "slack_sync")]
use slack_morphism::SlackUserProfile;

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
    let discovery = USBDeviceDiscovery::new().map_err(|e| e.to_string())?;
    let device = discovery.device().map_err(|e| e.to_string())?;
    #[cfg(feature = "tracing")]
    tracing::debug!("Found device: {}", device.id());

    let s = color.to_lowercase();
    match s.as_str() {
        "off" => device.turn_off().map_err(|e| e.to_string()),
        color_str => {
            if let Ok(parsed_color) = SolidColor::from_str(color_str) {
                let res = device
                    .set_solid_color(parsed_color.clone())
                    .map_err(|e| e.to_string());
                #[cfg(feature = "slack_sync")]
                {
                    let profile = color_to_profile(parsed_color);
                    let tokens = slack_api::retrieve_tokens(app.clone())?;
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
    let mut builder = tauri::Builder::default();

    builder = builder
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
        #[cfg(feature = "tracing")]
        tracing::info!("a new app instance was opened with {_args:?} and the deep link event was already triggered");
        // Focus this window
        let _ = app.get_webview_window("main")
            .expect("no main window")
            .set_focus();
    }));

    #[cfg(feature = "slack_sync")]
    {
        builder = builder
            .plugin(tauri_plugin_deep_link::init())
            .plugin(tauri_plugin_opener::init());
    }

    builder = builder
        .setup(move |app| {
            #[cfg(any(target_os = "linux", all(debug_assertions, windows)))]
            {
                app.deep_link().register_all()?;
            }

            #[cfg(feature = "slack_sync")]
            {
                let start_urls = app.deep_link().get_current()?;
                if let Some(urls) = start_urls {
                    // app was likely started by a deep link
                    println!("deep_link().get_current() URLs: {:?}", urls);
                }
                app.deep_link().on_open_url(|event| {
                    let urls = event.urls();
                    println!("deep_link().on_open_url() URLs: {:?}", &urls);
                    tauri::async_runtime::spawn(async move {
                        let url = urls[0].clone();
                        if let Ok(_tokens) = slack_api::try_parse_deep_link(url).await {
                            // TODO: Store the tokens
                            #[cfg(feature = "tracing")]
                            tracing::info!("Tokens were acquired successfully {:?} {:?}",
                            _tokens.user(), _tokens.bot()
                        );
                        }
                    });
                });
            }

            #[cfg(feature = "tracing")]
            tracing::info!("Starting Luxafor-ui");
            #[cfg(all(feature = "slack_sync", feature = "tracing"))]
            {
                tracing::debug!("SESSION_URL: {}", slack_api::SESSION_URL);
            }

            let app_config_dir = app
                .path()
                .app_config_dir()
                .expect("Failed to resolve app config dir");

            let settings_path = app_config_dir.join(SETTINGS_FILENAME);
            #[cfg(feature = "tracing")]
            tracing::info!("Settings path: {:?}", settings_path);

            // let handle = app.app_handle().clone();

            #[allow(unused_mut)]
            let mut settings_json_default = std::collections::HashMap::new();
            #[cfg(feature = "slack_sync")]
            settings_json_default.insert(
                "slack_tokens".to_string(),
                serde_json::to_value(slack_api::SlackApiTokens::default())?,
            );
            // TODO: Insert color/profile-mappings

            #[allow(unused)]
            let store = tauri_plugin_store::StoreBuilder::new(app, settings_path)
                .defaults(settings_json_default)
                .build()?;

            #[cfg(feature = "tracing")]
            tracing::debug!("Store contents:\n{:#?}", store.entries());

            let about_meta = AboutMetadataBuilder::new()
                .name(Some(PKG_NAME))
                .authors(Some(vec![AUTHOR.into()]))
                .comments(Some(COMMENTS))
                .copyright(Some(COPYRIGHT))
                .icon(Some(
                    app.app_handle().default_window_icon().unwrap().clone(),
                ))
                .build();

            let about_i = PredefinedMenuItem::about(app, Some("About"), Some(about_meta))?;

            let quit_i = MenuItemBuilder::with_id("quit", "Quit").build(app)?;

            let luxafor_ui_i = MenuItemBuilder::with_id("luxafor_ui", PKG_NAME).build(app)?;

            #[cfg(feature = "slack_sync")]
            let add_to_slack_i =
                MenuItemBuilder::with_id("add_to_slack", "Add to Slack").build(app)?;

            let menu = MenuBuilder::new(app)
                .items(&[
                    &luxafor_ui_i,
                    &about_i,
                    #[cfg(feature = "slack_sync")]
                    &PredefinedMenuItem::separator(app)?,
                    #[cfg(feature = "slack_sync")]
                    &add_to_slack_i,
                    &PredefinedMenuItem::separator(app)?,
                    &quit_i,
                ])
                .build()?;

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .tooltip(PKG_NAME)
                .show_menu_on_left_click(true)
                .icon(app.app_handle().default_window_icon().unwrap().clone())
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
                        let store_handle = app.app_handle().clone();
                        tauri::async_runtime::spawn(async {
                            let add_to_slack_task = async {
                                let auth_session = slack_api::SlackAuthSession::new()
                                    .init_session()
                                    .await
                                    .unwrap_or_else(|e| {
                                        panic!("Could not initialize Slack Auth session: {}", e)
                                    });

                                tauri_plugin_opener::open_url(
                                    auth_session.authorize_url(),
                                    None::<&str>,
                                )
                                .unwrap();
                                loop {
                                    match auth_session.poll_status().await {
                                        Ok(slack_api::PollingSessionResponse {
                                            status: slack_api::PollStatus::Pending,
                                            ..
                                        }) => {
                                            #[cfg(feature = "tracing")]
                                            tracing::debug!("Authorization pending...");
                                            //  Wait a bit before polling again
                                            tokio::time::sleep(std::time::Duration::from_secs(5))
                                                .await;
                                        }
                                        Ok(slack_api::PollingSessionResponse {
                                            status: slack_api::PollStatus::Ok,
                                            tokens,
                                            ..
                                        }) => {
                                            #[cfg(feature = "tracing")]
                                            tracing::info!("Authorization successful!");

                                            if let Some(tokens) = tokens {
                                                #[cfg(feature = "tracing")]
                                                tracing::debug!("Received tokens:\n{:#?}", tokens);

                                                slack_api::store_tokens(store_handle, tokens)
                                                    .unwrap_or_else(|e| {
                                                        panic!("Could not store tokens: {}", e);
                                                    });
                                            }
                                            break;
                                        }
                                        _ => {} // Ignore all other statuses
                                    }
                                }
                            };
                            if tokio::time::timeout(
                                std::time::Duration::from_mins(5),
                                add_to_slack_task,
                            )
                            .await
                            .is_err()
                            {
                                // TODO: Notify user of timeout
                                #[cfg(feature = "tracing")]
                                tracing::error!("Add to Slack task timed out!");
                            }
                        });
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
                .build(app)?;
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
        .invoke_handler(tauri::generate_handler![set_light_color,]);

    builder.run(tauri::generate_context!())?;

    Ok(())
}
