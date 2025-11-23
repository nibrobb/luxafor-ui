use serde::{Deserialize, Serialize};
use slack_morphism::prelude::*;

#[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
pub async fn status_set(
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

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(
        skip_all,
        fields(status_text = ?profile.status_text, emoji = ?profile.status_emoji)
    )
)]
pub async fn slack_set_profile(
    profile: SlackUserProfile,
    tokens: SlackApiTokens,
) -> Result<(), String> {
    if let Some(token) = tokens.user_token {
        if !token.0.starts_with("xoxp-") {
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
pub struct SlackApiTokens {
    user_token: Option<SlackApiTokenValue>,
    bot_token: Option<SlackApiTokenValue>,
}

impl SlackApiTokens {
    pub fn new(
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
