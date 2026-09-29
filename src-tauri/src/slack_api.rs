use crate::SETTINGS_FILENAME;
use serde::{Deserialize, Serialize};
use slack_morphism::prelude::*;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_store::StoreExt;

#[cfg(feature = "tracing")]
use tracing::{debug, error, instrument};

pub(crate) const SLACK_OAUTH_URL: &str = env!("SLACK_OAUTH_URL");

#[cfg_attr(feature = "tracing", instrument(skip_all))]
pub(crate) async fn status_set(
    profile: SlackUserProfile,
    user_token: SlackApiTokenValue,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    #[cfg(feature = "tracing")]
    debug!(
        "status_text: {:?}, status_emoji: {:?}",
        profile.status_text,
        profile.status_emoji
    );

    let client = SlackClient::new(SlackClientHyperConnector::new()?);
    let user_token = SlackApiToken::new(user_token);

    match client
        .run_in_session(&user_token, |session| {
            let profile_clone = profile.clone();
            async move {
                let status_request = SlackApiUsersProfileSetRequest::new(profile_clone);
                session.users_profile_set(&status_request).await
            }
        })
        .await
    {
        Ok(_something) => Ok(()),
        Err(e) => {
            #[cfg(feature = "tracing")]
            error!("Error setting status: {:#?}", e);
            Err(Box::new(e))
        }
    }
}

fn validate_user_token(token: &SlackApiTokenValue) -> bool {
    token.0.starts_with("xoxp-")
}

#[cfg_attr(
    feature = "tracing",
    instrument(
        skip_all,
        fields(status_text = ?profile.status_text, emoji = ?profile.status_emoji)
    )
)]
pub(crate) async fn slack_set_profile(
    profile: SlackUserProfile,
    tokens: SlackApiTokens,
) -> Result<(), String> {
    #[cfg(feature = "tracing")]
    debug!(
        "Attempting Slack profile update: status_text={:?}, status_emoji={:?}",
        profile.status_text,
        profile.status_emoji
    );

    if let Some(token) = tokens.user_token {
        if !validate_user_token(&token) {
            #[cfg(feature = "tracing")]
            error!("user_token is not a valid user token: {:?}", token);
            return Err("user_token is not a valid user token".to_string());
        }
        #[cfg(feature = "tracing")]
        debug!("Valid user token detected, dispatching Slack status update");
        if let Err(e) =
            tauri::async_runtime::spawn(
                async move { status_set(profile.clone(), token.clone()).await },
            )
            .await
        {
            Err(e.to_string())
        } else {
            #[cfg(feature = "tracing")]
            debug!("Slack status update task completed successfully");
            Ok(())
        }
    } else {
        #[cfg(feature = "tracing")]
        tracing::warn!("user_token not found; skipping Slack status update");
        Err("user_token not found".to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct SlackApiTokens {
    #[serde(default)]
    pub user_token: Option<SlackApiTokenValue>,
    #[serde(default)]
    pub bot_token: Option<SlackApiTokenValue>,
}

impl SlackApiTokens {
    pub(crate) fn new(
        user_token: Option<SlackApiTokenValue>,
        bot_token: Option<SlackApiTokenValue>,
    ) -> Self {
        SlackApiTokens {
            user_token,
            bot_token,
        }
    }
    pub(crate) fn user_token(&self) -> Option<SlackApiToken> {
        self.user_token
            .as_ref()
            .map(|token| SlackApiToken::new(token.clone()))
    }
    pub(crate) fn bot_token(&self) -> Option<SlackApiToken> {
        self.bot_token
            .as_ref()
            .map(|token| SlackApiToken::new(token.clone()))
    }
    pub(crate) fn has_any(&self) -> bool {
        self.user_token.is_some() || self.bot_token.is_some()
    }
}

impl TryFrom<serde_json::Value> for SlackApiTokens {
    type Error = serde_json::Error;

    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        let user_token = value
            .get("user_token")
            .and_then(|v| v.as_str())
            .map(|s| s.into());
        let bot_token = value
            .get("bot_token")
            .and_then(|v| v.as_str())
            .map(|s| s.into());
        Ok(SlackApiTokens::new(user_token, bot_token))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub(crate) struct SlackProfileMapEntry {
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub emoji: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct SlackSettings {
    #[serde(default, rename = "slack_tokens")]
    pub slack_tokens: SlackApiTokens,
    #[serde(default, rename = "slack_status_map")]
    pub slack_status_map: HashMap<String, SlackProfileMapEntry>,
}

impl SlackSettings {
    pub(crate) const TOKENS_KEY: &str = "slack_tokens";
    pub(crate) const STATUS_MAP_KEY: &str = "slack_status_map";

    pub(crate) fn defaults() -> Self {
        Self {
            slack_tokens: SlackApiTokens::default(),
            slack_status_map: Self::default_status_map(),
        }
    }

    pub(crate) fn default_status_map() -> HashMap<String, SlackProfileMapEntry> {
        HashMap::from([
            (
                "red".to_string(),
                SlackProfileMapEntry {
                    status: Some("Busy".to_string()),
                    emoji: Some(":no_entry:".to_string()),
                },
            ),
            (
                "green".to_string(),
                SlackProfileMapEntry {
                    status: Some("Available".to_string()),
                    emoji: None,
                },
            ),
            (
                "yellow".to_string(),
                SlackProfileMapEntry {
                    status: Some("Away".to_string()),
                    emoji: Some(":warning:".to_string()),
                },
            ),
            (
                "blue".to_string(),
                SlackProfileMapEntry {
                    status: Some("In a meeting".to_string()),
                    emoji: Some(":blueberries:".to_string()),
                },
            ),
            (
                "cyan".to_string(),
                SlackProfileMapEntry {
                    status: Some("Focus time".to_string()),
                    emoji: Some(":raccoon:".to_string()),
                },
            ),
            (
                "white".to_string(),
                SlackProfileMapEntry {
                    status: Some("Online".to_string()),
                    emoji: None,
                },
            ),
            (
                "magenta".to_string(),
                SlackProfileMapEntry {
                    status: Some("On a call".to_string()),
                    emoji: Some(":phone:".to_string()),
                },
            ),
        ])
    }

    fn parse_tokens(value: serde_json::Value) -> Option<SlackApiTokens> {
        serde_json::from_value::<SlackApiTokens>(value.clone()).ok().or_else(|| {
            let object = value.as_object()?;
            let user_token = object
                .get("user_token")
                .and_then(serde_json::Value::as_str)
                .map(|token| SlackApiTokenValue(token.to_string()));
            let bot_token = object
                .get("bot_token")
                .and_then(serde_json::Value::as_str)
                .map(|token| SlackApiTokenValue(token.to_string()));
            Some(SlackApiTokens::new(user_token, bot_token))
        })
    }

    pub(crate) fn load(app: &AppHandle) -> Result<Self, String> {
        let store_path = resolve_store_path(app)?;
        if !store_path.exists() {
            return Ok(Self::defaults());
        }

        let store = app
            .get_store(store_path)
            .ok_or("Could not get store".to_string())?;
        store
            .reload()
            .map_err(|e| format!("Reload store failed: {}", e))?;

        let slack_tokens = store
            .get(Self::TOKENS_KEY)
            .and_then(Self::parse_tokens)
            .unwrap_or_default();

        let slack_status_map = store
            .get(Self::STATUS_MAP_KEY)
            .and_then(|value| {
                serde_json::from_value::<HashMap<String, SlackProfileMapEntry>>(value).ok()
            })
            .unwrap_or_default();

        Ok(Self {
            slack_tokens,
            slack_status_map,
        })
    }

    pub(crate) fn save(&self, app: &AppHandle) -> Result<(), String> {
        let store_path = resolve_store_path(app)?;
        if let Some(parent_dir) = store_path.parent() {
            std::fs::create_dir_all(parent_dir).map_err(|e| format!("Could not create settings dir: {}", e))?;
        }

        let store = app
            .get_store(store_path)
            .ok_or("Could not get store".to_string())?;
        store.set(
            Self::TOKENS_KEY,
            serde_json::to_value(&self.slack_tokens).map_err(|e| e.to_string())?,
        );
        store.set(
            Self::STATUS_MAP_KEY,
            serde_json::to_value(&self.slack_status_map).map_err(|e| e.to_string())?,
        );
        store.save().map_err(|e| e.to_string())
    }

    pub(crate) fn warn_missing_tokens(app: &AppHandle) {
        let message = "Slack is enabled, but no Slack tokens were found in settings.json. Use the Add to Slack action from the tray menu to install the companion Slack app and finish setup.".to_string();
        eprintln!("Warning: {message}");
        let _ = app.emit("slack_warning", message);
    }

    pub(crate) fn warn_missing_status_map(app: &AppHandle) {
        let message = "Slack tokens were found, but no status mappings are configured in settings.json. Add entries under slack_status_map to enable Slack status updates.".to_string();
        eprintln!("Warning: {message}");
        let _ = app.emit("slack_warning", message);
    }

    pub(crate) fn profile_for_color(&self, color: &luxafor::SolidColor) -> SlackUserProfile {
        let key = color.to_string();
        let profile = self.slack_status_map.get(&key).cloned().unwrap_or_default();

        let mut slack_profile = SlackUserProfile::new();
        if let Some(status) = profile.status.filter(|value| !value.trim().is_empty()) {
            slack_profile = slack_profile.with_status_text(status);
        }
        if let Some(emoji) = profile.emoji.filter(|value| !value.trim().is_empty()) {
            slack_profile = slack_profile.with_status_emoji(SlackEmoji(emoji));
        }
        slack_profile
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn deserializes_dummy_settings() {
        let input = json!({
            "slack_tokens": {
                "user_token": "xoxp-1234567890",
                "bot_token": "xoxb-0987654321"
            },
            "slack_status_map": {
                "red": {
                    "status": "On a call",
                    "emoji": ":red_circle:"
                },
                "green": {
                    "status": "Available",
                    "emoji": null
                },
                "blue": {}
            }
        });

        let settings: SlackSettings = serde_json::from_value(input).unwrap();

        assert!(settings.slack_tokens.has_any());
        assert_eq!(settings.slack_tokens.user_token.as_ref().unwrap().value(), "xoxp-1234567890");
        assert_eq!(settings.slack_tokens.bot_token.as_ref().unwrap().value(), "xoxb-0987654321");
        assert_eq!(settings.slack_status_map.get("red").unwrap().status.as_deref(), Some("On a call"));
        assert_eq!(settings.slack_status_map.get("green").unwrap().emoji, None);
        assert!(settings.slack_status_map.get("blue").unwrap().status.is_none());
    }

    #[test]
    fn empty_settings_are_allowed() {
        let settings: SlackSettings = serde_json::from_value(json!({})).unwrap();
        assert!(!settings.slack_tokens.has_any());
        assert!(settings.slack_status_map.is_empty());
    }
}

/// Resolves the path to the settings.json file in the app's config directory.
fn resolve_store_path(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    app.path()
        .app_config_dir()
        .map_err(|e| e.to_string())
        .map(|path| path.join(SETTINGS_FILENAME))
}

/// Retrieves the SlackApiTokens from the store.
pub(crate) fn retrieve_tokens(app: AppHandle) -> Result<SlackApiTokens, String> {
    SlackSettings::load(&app).map(|settings| settings.slack_tokens)
}

impl AsRef<SlackApiTokens> for SlackApiTokens {
    fn as_ref(&self) -> &Self {
        self
    }
}

/// Stores the SlackApiTokens in the store (resolved settings.json).
pub(crate) fn store_tokens<T, U>(app: AppHandle, tokens: T) -> Result<(), String>
where
    T: AsRef<U>,
    U: Serialize,
{
    let settings = SlackSettings::load(&app).unwrap_or_else(|_| SlackSettings::defaults());
    let mut updated_settings = settings;
    updated_settings.slack_tokens = serde_json::from_value(
        serde_json::to_value(tokens.as_ref()).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    updated_settings.save(&app)
}

pub(crate) enum DeepLinkParseError {
    ParseError,
    DomainError,
    MissingUserToken,
    MissingBotToken,
    IncorrectQueryString,
    APITestFailed(String),
}

impl Display for DeepLinkParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use DeepLinkParseError::*;

        let message = match *self {
            ParseError => "ParseError".to_string(),
            DomainError => "DomainError".to_string(),
            MissingUserToken => "MissingUserToken".to_string(),
            MissingBotToken => "MissingBotToken".to_string(),
            IncorrectQueryString => "IncorrectQueryString".to_string(),
            APITestFailed(ref msg) => format!("APITestFailed({msg})"),
        };

        f.write_fmt(format_args!("DeepLinkParseError::{}", message))
    }
}
async fn test_token<SC>(client: &SlackClient<SC>, token: &SlackApiToken) -> Result<(), String>
where
    SC: SlackClientHttpConnector + Send + Sync,
{
    match client
        .run_in_session(token, |s| async move {
            s.api_test(&SlackApiTestRequest::new()).await
        })
        .await
        .map_err(|e| e.to_string())
    {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}
async fn test_api(tokens: &SlackApiTokens) -> Result<(), String> {
    let connector = SlackClientHyperConnector::new().unwrap();
    let client = SlackClient::new(connector);

    if let Some(user_token) = tokens.user_token() {
        test_token(&client, &user_token).await?;
    }
    if let Some(bot_token) = tokens.bot_token() {
        test_token(&client, &bot_token).await?;
    }

    Ok(())
}

pub(crate) async fn try_parse_deep_link(
    url: tauri::Url,
) -> Result<SlackApiTokens, DeepLinkParseError> {
    use DeepLinkParseError::*;

    if let Some(auth) = url.domain() {
        if auth != "auth" {
            return Err(DomainError);
        }
        let mut query_pairs = url.query_pairs();
        if let Some((key1, value1)) = query_pairs.next() {
            if key1 != "user_token" {
                Err(MissingUserToken)
            } else if let Some((key2, value2)) = query_pairs.next() {
                if key2 != "bot_token" {
                    Err(MissingBotToken)
                } else {
                    let user_token = SlackApiToken::new(SlackApiTokenValue(value1.into()));
                    let bot_token = SlackApiToken::new(SlackApiTokenValue(value2.into()));
                    let tokens = SlackApiTokens::new(
                        Some(user_token.token_value),
                        Some(bot_token.token_value),
                    );
                    if let Err(e) = test_api(&tokens).await {
                        Err(APITestFailed(e))
                    } else {
                        Ok(tokens)
                    }
                }
            } else {
                Err(IncorrectQueryString)
            }
        } else {
            Err(IncorrectQueryString)
        }
    } else {
        Err(ParseError)
    }
}
