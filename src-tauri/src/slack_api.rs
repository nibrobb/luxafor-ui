use slack_morphism::prelude::*;
use tracing::{debug, error};

#[allow(unused)]
pub(crate) async fn status_set(
    profile: SlackUserProfile,
    user_token: SlackApiTokenValue,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    debug!(
        "status_text: {:?}, status_emoji: {:?}",
        profile.status_text, profile.status_emoji
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
            error!("Error setting status: {:#?}", e);
            Err(Box::new(e))
        }
    }
}
