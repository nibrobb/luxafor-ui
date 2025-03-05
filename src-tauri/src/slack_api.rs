use http_body_util::combinators::BoxBody;
use http_body_util::BodyExt;
use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use rvstruct::ValueStruct;
use slack_morphism::prelude::*;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;
use tracing::log::debug;

pub const INSTALL_URL: &str = "http://localhost:8080/auth/install";

pub fn config_env_var(name: &str) -> Result<String, String> {
    std::env::var(name).map_err(|e| format!("{}: {}", name, e))
}

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
        println!("{:#?}", err);
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

        println!("{:#?}", resp);
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
        println!("{:#?}", event);
        Ok(())
    }

    async fn interaction_events_function(
        event: SlackInteractionEvent,
        _client: Arc<SlackHyperClient>,
        _states: SlackClientEventsUserState,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("{:#?}", event);
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

        println!("{:#?}", event);
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

    info!("Server is listening on http://{}", &addr);

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
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}

pub async fn send_status(
    color: String, /* luxafor::SolidColor */
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Read `user_token` from `store.json` file, then use that with Slack's `users.profile.set` API to set some status
    let file = std::fs::File::open("store.json")?;
    let reader = std::io::BufReader::new(file);
    let data: serde_json::Value = serde_json::from_reader(reader)?;
    let user_token: String = data["user_token"]
        .as_str()
        .expect("user_token not found in store.json")
        .to_string();
    debug!("User token: {}", user_token);

    let client = SlackClient::new(SlackClientHyperConnector::new()?);
    let token = SlackApiToken::new(SlackApiTokenValue::new(user_token));

    client
        .run_in_session(&token, |session| {
            let color_clone = color.clone();
            async move {
                let profile: SlackUserProfile = match color_clone.as_str() {
                    // TODO: use enum or something to match colors
                    "yellow" => SlackUserProfile::new()
                        .with_status_emoji(":slack:".into())
                        .with_status_text("Slacking".into()),
                    _ => {
                        debug!("Unknown color: {}", color_clone);
                        SlackUserProfile::new()
                            .with_status_emoji("".into())
                            .with_status_text("".into())
                    }
                };
                let status_request = SlackApiUsersProfileSetRequest::new(profile);
                session.users_profile_set(&status_request).await
                // debug!("{:#?}", response);
                // response
            }
        })
        .await?;

    Ok(())
}
