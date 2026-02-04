use crate::SETTINGS_FILENAME;
use serde::{Deserialize, Serialize};
use slack_morphism::prelude::*;
use std::fmt::{Display, Formatter};
use tauri::{AppHandle, Manager};
use tauri_plugin_store::StoreExt;

pub(crate) const SLACK_OAUTH_URL: &str = env!("SLACK_OAUTH_URL");

#[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
pub(crate) async fn status_set(
    profile: SlackUserProfile,
    user_token: SlackApiTokenValue,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    #[cfg(feature = "tracing")]
    tracing::debug!(
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
            tracing::error!("Error setting status: {:#?}", e);
            Err(Box::new(e))
        }
    }
}

fn validate_user_token(token: &SlackApiTokenValue) -> bool {
    token.0.starts_with("xoxp-")
}

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(
        skip_all,
        fields(status_text = ?profile.status_text, emoji = ?profile.status_emoji)
    )
)]
pub(crate) async fn slack_set_profile(
    profile: SlackUserProfile,
    tokens: SlackApiTokens,
) -> Result<(), String> {
    if let Some(token) = tokens.user_token {
        if !validate_user_token(&token) {
            #[cfg(feature = "tracing")]
            tracing::error!("user_token is not a valid user token: {:?}", token);
            return Err("user_token is not a valid user token".to_string());
        }
        if let Err(e) =
            tauri::async_runtime::spawn(
                async move { status_set(profile.clone(), token.clone()).await },
            )
            .await
        {
            Err(e.to_string())
        } else {
            Ok(())
        }
    } else {
        Err("user_token not found".to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct SlackApiTokens {
    user_token: Option<SlackApiTokenValue>,
    bot_token: Option<SlackApiTokenValue>,
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
    #[allow(unused)]
    pub(crate) fn user(&self) -> Option<&SlackApiTokenValue> {
        self.user_token.as_ref()
    }
    #[allow(unused)]
    pub(crate) fn bot(&self) -> Option<&SlackApiTokenValue> {
        self.bot_token.as_ref()
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

/// Resolves the path to the settings.json file in the app's config directory.
fn resolve_store_path(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    app.path()
        .app_config_dir()
        .map_err(|e| e.to_string())
        .map(|path| path.join(SETTINGS_FILENAME))
}

/// Retrieves the SlackApiTokens from the store.
pub(crate) fn retrieve_tokens(app: AppHandle) -> Result<SlackApiTokens, String> {
    let store_path = resolve_store_path(app.app_handle())?;
    let store = app
        .get_store(store_path)
        .ok_or("Could not get store".to_string())?;
    store
        .reload()
        .map_err(|e| format!("Reload store failed: {}", e))?;

    match store
        .get("slack_tokens")
        .and_then(|value| value.try_into().ok())
    {
        Some(tokens) => Ok(tokens),
        None => {
            Err("Key `slack_tokens` not found in `settings.json` or could not parse".to_string())
        }
    }
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
    let store_path = resolve_store_path(app.app_handle())?;
    let store = app
        .get_store(store_path)
        .ok_or("Could not get store".to_string())?;
    store.set(
        "slack_tokens",
        serde_json::to_value(tokens.as_ref()).map_err(|e| e.to_string())?,
    );
    store.save().map_err(|e| e.to_string())
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

async fn test_api(token: &SlackApiToken) -> Result<(), String> {
    let connector = SlackClientHyperConnector::new().unwrap();
    let client = SlackClient::new(connector);
    let res = client
        .run_in_session(token, |s| async move {
            s.api_test(&SlackApiTestRequest::new()).await
        })
        .await
        .map_err(|e| e.to_string());
    match res {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
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
                    // TODO: Test both tokens
                    if let Err(e) = test_api(&user_token).await {
                        Err(APITestFailed(e))
                    } else {
                        Ok(SlackApiTokens::new(
                            Some(user_token.token_value),
                            Some(bot_token.token_value),
                        ))
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
