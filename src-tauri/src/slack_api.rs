use crate::{SESSION_STATUS_URL, SESSION_URL, SETTINGS_FILENAME};
use serde::{Deserialize, Serialize};
use slack_morphism::prelude::*;
use tauri::{AppHandle, Manager};
use tauri_plugin_store::StoreExt;

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

// {
//     "session_id": "b1d6a3489910c4612c13023c0f1b99a2",
//     "session_secret": "12a6bae73b74f2df40da5a04fd271f3e",
//     "authorize_url": "http://preview.nibrobb.dev//oauth/slack?session_id=b1d6a3489910c4612c13023c0f1b99a2&session_secret=12a6bae73b74f2df40da5a04fd271f3e"
// }
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SlackAuthSession {
    #[serde(skip)]
    client: reqwest::Client,
    session_id: Option<String>,
    session_secret: Option<String>,
    authorize_url: Option<reqwest::Url>,
}
#[derive(Deserialize, Serialize, Clone, Debug)]
pub(crate) enum PollStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "invalid")]
    Invalid,
    #[serde(rename = "ok")]
    Ok,
    #[serde(rename = "error")]
    Error,
}
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct PollingSessionResponse {
    pub(crate) status: PollStatus,
    pub(crate) tokens: Option<SlackApiTokens>,
}

impl SlackAuthSession {
    pub(crate) fn authorize_url(&self) -> &reqwest::Url {
        self.authorize_url.as_ref().unwrap()
    }
    pub(crate) fn session_id_secret(&self) -> (&str, &str) {
        (
            self.session_id.as_ref().unwrap(),
            self.session_secret.as_ref().unwrap(),
        )
    }
    pub(crate) fn new() -> Self {
        static APP_USER_AGENT: &str =
            concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);
        Self {
            client: reqwest::Client::builder()
                .user_agent(APP_USER_AGENT)
                .build()
                .unwrap(),
            session_id: None,
            session_secret: None,
            authorize_url: None,
        }
    }
    pub(crate) async fn init_session(
        &self,
    ) -> Result<SlackAuthSession, Box<dyn std::error::Error>> {
        let response = self
            .client
            .request(reqwest::Method::POST, SESSION_URL)
            .header(
                "x-vercel-protection-bypass",
                "9atIb6T13C4Exyo9RM8TW4kx0mKZymcU",
            )
            .header("Content-Type", "application/json")
            .send()
            .await?;

        if response.status() != reqwest::StatusCode::OK {
            return Err("Could not get session".into());
        }

        let res_json = response.json::<SlackAuthSession>().await?;
        #[cfg(feature = "tracing")]
        tracing::debug!("InitSessionResponse:\n{:#?}", res_json);
        Ok(res_json)
    }
    pub(crate) async fn poll_status(&self) -> PollingSessionResponse {
        self.client
            .request(
                reqwest::Method::GET,
                format!(
                    "{}?session_id={}&session_secret={}",
                    SESSION_STATUS_URL,
                    self.session_id.as_ref().unwrap(),
                    self.session_secret.as_ref().unwrap()
                ),
            )
            .header(
                "x-vercel-protection-bypass",
                "9atIb6T13C4Exyo9RM8TW4kx0mKZymcU",
            )
            .header("Content-Type", "application/json")
            .send()
            .await
            .unwrap()
            .json::<PollingSessionResponse>()
            .await
            .unwrap()
    }
}

fn resolve_store_path(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    app.path()
        .app_config_dir()
        .map_err(|e| e.to_string())
        .map(|path| path.join(SETTINGS_FILENAME))
}

pub(crate) fn retrieve_tokens(app: AppHandle) -> Result<SlackApiTokens, String> {
    let store_path = resolve_store_path(app.app_handle())?;
    let store = app
        .get_store(store_path)
        .ok_or("Could not get store".to_string())?;
    store.reload().expect("Reload store failed");

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

pub(crate) fn store_tokens(app: AppHandle, tokens: &SlackApiTokens) -> Result<(), String> {
    let store_path = resolve_store_path(app.app_handle())?;
    let store = app
        .get_store(store_path)
        .ok_or("Could not get store".to_string())?;
    store.set(
        "slack_tokens",
        serde_json::to_value(tokens).map_err(|e| e.to_string())?,
    );
    store.save().map_err(|e| e.to_string())
}
