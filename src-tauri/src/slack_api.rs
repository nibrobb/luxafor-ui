#[cfg(feature = "slack_oauth")]
use {
    http_body_util::combinators::BoxBody,
    http_body_util::BodyExt,
    http_body_util::Full,
    hyper::body::{Bytes, Incoming},
    hyper::service::service_fn,
    hyper::{Request, Response},
    hyper_util::rt::TokioIo,
    rvstruct::ValueStruct,
    std::convert::Infallible,
    std::sync::Arc,
    tokio::net::TcpListener,
};

#[allow(unused_imports)]
use {
    slack_morphism::prelude::*,
    tracing::{debug, error},
};

#[cfg(feature = "slack_oauth")]
#[allow(unused)]
pub const INSTALL_URL: &str = "http://localhost:8080/auth/install";

#[allow(unused)]
pub fn config_env_var(name: &str) -> Result<String, String> {
    std::env::var(name).map_err(|e| format!("{}: {}", name, e))
}

//noinspection HttpUrlsUsage
#[cfg(feature = "slack_oauth")]
#[allow(unused)]
/// OAuth 2.0 flow for "Add to Slack"
/// To use "Add to Slack" the redirect URI in the Slack app must be an HTTPS URL
/// https://api.slack.com/authentication/oauth-v2
/// The "Add to Slack" link is like this: https://slack.com/oauth/v2/authorize?client_id=CLIENT_ID&scope=BOT_SCOPE&user_scope=USER_SCOPE
pub async fn setup_oauth() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    fn test_error_handler(
        err: Box<dyn std::error::Error + Send + Sync>,
        _client: Arc<SlackHyperClient>,
        _states: SlackClientEventsUserState,
    ) -> HttpStatusCode {
        debug!("{:#?}", err);
        // Defines what we return Slack server
        HttpStatusCode::BAD_REQUEST
    }

    async fn oauth_install_function(
        resp: SlackOAuthV2AccessTokenResponse,
        _client: Arc<SlackHyperClient>,
        _states: SlackClientEventsUserState,
    ) {
        debug!("Bot access token:\t{}", resp.access_token.value());

        if let Some(ref user_token) = resp.authed_user.access_token {
            debug!("User access token\t{}", user_token.value());
        };

        debug!("{:#?}", resp);
        use std::io::Write;
        let mut file = std::fs::File::create("store.json").expect("create 'store.json' failed");
        use serde_json::json;
        let json_data = json!({
            "bot_token": resp.access_token.value(),
            "user_token": resp.authed_user.access_token.as_ref().map(|v| v.value()),
        });
        debug!("Writing tokens to file:\n\n{:?}", json_data);
        file.write_all(json_data.to_string().as_bytes())
            .expect("write 'store.json' failed");
    }

    async fn push_events_function(
        event: SlackPushEvent,
        _client: Arc<SlackHyperClient>,
        _states: SlackClientEventsUserState,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        debug!("{:#?}", event);
        Ok(())
    }

    async fn interaction_events_function(
        event: SlackInteractionEvent,
        _client: Arc<SlackHyperClient>,
        _states: SlackClientEventsUserState,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        debug!("{:#?}", event);
        Ok(())
    }

    async fn command_events_function(
        event: SlackCommandEvent,
        _client: Arc<SlackHyperClient>,
        _states: SlackClientEventsUserState,
    ) -> Result<SlackCommandEventResponse, Box<dyn std::error::Error + Send + Sync>> {
        let token_value: SlackApiTokenValue = config_env_var("SLACK_BOT_TOKEN")?.into();
        let token: SlackApiToken = SlackApiToken::new(token_value);
        let session = _client.open_session(&token);

        session
            .api_test(&SlackApiTestRequest::new().with_foo("Test".into()))
            .await?;

        debug!("{:#?}", event);
        Ok(SlackCommandEventResponse::new(
            SlackMessageContent::new().with_text("Working on it".into()),
        ))
    }

    async fn default_route(
        req: Request<Incoming>,
    ) -> Result<Response<BoxBody<Bytes, Infallible>>, Box<dyn std::error::Error + Send + Sync>>
    {
        Response::builder()
            .body(
                Full::new(
                    format!(
                        "Hey, this is a default user route handler\nYour request was:\n\n{:#?}",
                        req
                    )
                    .into(),
                )
                .boxed(),
            )
            .map_err(|e| e.into())
    }

    let client = Arc::new(SlackClient::new(SlackClientHyperConnector::new()?));

    // The address we listen on (must match what is set in the redirect URL thing in api.slack.com)
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 8080));

    let oauth_listener_config = Arc::new(SlackOAuthListenerConfig::new(
        SlackClientId(config_env_var("SLACK_CLIENT_ID")?),
        SlackClientSecret(config_env_var("SLACK_CLIENT_SECRET")?),
        "users.profile:read".into(),
        "users.profile:read,users.profile:write".into(),
        config_env_var("REDIRECT_HOST")?,
    ));

    let push_events_config = Arc::new(SlackPushEventsListenerConfig::new(
        config_env_var("SLACK_SIGNING_SECRET")?.into(),
    ));
    let interactions_events_config = Arc::new(SlackInteractionEventsListenerConfig::new(
        config_env_var("SLACK_SIGNING_SECRET")?.into(),
    ));
    let command_events_config = Arc::new(SlackCommandEventsListenerConfig::new(
        config_env_var("SLACK_SIGNING_SECRET")?.into(),
    ));

    let listener_environment = Arc::new(
        SlackClientEventsListenerEnvironment::new(client.clone())
            .with_error_handler(test_error_handler),
    );

    let listener = TcpListener::bind(&addr).await?;

    debug!("Server is listening on http://{}", &addr);

    loop {
        let (tcp, _) = listener.accept().await?;
        let io = TokioIo::new(tcp);

        let thread_oauth_config = oauth_listener_config.clone();
        let thread_push_events_config = push_events_config.clone();
        let thread_interaction_events_config = interactions_events_config.clone();
        let thread_command_events_config = command_events_config.clone();
        let listener = SlackClientEventsHyperListener::new(listener_environment.clone());
        let routes = chain_service_routes_fn(
            listener.oauth_service_fn(thread_oauth_config, oauth_install_function),
            chain_service_routes_fn(
                listener.push_events_service_fn(thread_push_events_config, push_events_function),
                chain_service_routes_fn(
                    listener.interaction_events_service_fn(
                        thread_interaction_events_config,
                        interaction_events_function,
                    ),
                    chain_service_routes_fn(
                        listener.command_events_service_fn(
                            thread_command_events_config,
                            command_events_function,
                        ),
                        default_route,
                    ),
                ),
            ),
        );

        tokio::task::spawn(async move {
            if let Err(err) = hyper::server::conn::http1::Builder::new()
                .serve_connection(io, service_fn(routes))
                .await
            {
                error!("Error serving connection: {:?}", err);
            }
        });
    }
}

#[cfg(feature = "slack_sync")]
#[tracing::instrument(skip_all)]
pub async fn status_set(
    profile: SlackUserProfile,
    user_token: SlackApiToken,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    debug!(
        "status_text: {:?}, status_emoji: {:?}",
        profile.status_text, profile.status_emoji
    );

    // Read `user_token` from `store.json` file, then use that with Slack's `users.profile.set` API to set some status
    // let file = match std::fs::File::open("store.json") {
    //     Ok(file) => file,
    //     Err(e) => {
    //         error!("store.json not found");
    //         return Err(Box::new(e))
    //     }
    // };
    // let reader = std::io::BufReader::new(file);
    // let data: serde_json::Value = match serde_json::from_reader(reader) {
    //     Ok(v) => v,
    //     Err(e) => {
    //         warn!("Error reading store.json: {}", e);
    //         return Err(Box::new(e));
    //     }
    // };
    // let user_token: String = match data["user_token"].as_str() {
    //     Some(v) => v.to_string(),
    //     None => {
    //         warn!("user_token not found in store.json");
    //         return Err(Box::new(std::io::Error::new(
    //             std::io::ErrorKind::NotFound,
    //             "user_token not found in store.json",
    //         )));
    //     }
    // };

    let client = SlackClient::new(SlackClientHyperConnector::new()?);
    // let token = SlackApiToken::new(SlackApiTokenValue::new(user_token));

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
