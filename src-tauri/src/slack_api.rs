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

pub fn config_env_var(name: &str) -> Result<String, String> {
    std::env::var(name).map_err(|e| format!("{}: {}", name, e))
}

/// Installing with OAuth (Add to Slack)
/// The user clicks "Add to Slack" in GitHub repo (perhaps)
/// The user is redirected to Slack OAuth page
/// The user authorizes the app
/// The user is redirected back to the app (aka. localhost)
/// The app receives the OAuth code
/// The app exchanges the code for an access token
/// The app receives the access token
/// The app can now make API calls on behalf of the user
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

    async fn test_oauth_install_function(
        resp: SlackOAuthV2AccessTokenResponse,
        _client: Arc<SlackHyperClient>,
        _states: SlackClientEventsUserState,
    ) {
        debug!("Bot access token:\t{}", resp.access_token.value());

        if let Some(ref user_token) = resp.authed_user.access_token {
            debug!("User access token\t{}", user_token.value());
        };

        println!("{:#?}", resp);

        // Save the tokens in Tauri's Store or something
    }

    async fn test_push_events_function(
        event: SlackPushEvent,
        _client: Arc<SlackHyperClient>,
        _states: SlackClientEventsUserState,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("{:#?}", event);
        Ok(())
    }

    async fn test_interaction_events_function(
        event: SlackInteractionEvent,
        _client: Arc<SlackHyperClient>,
        _states: SlackClientEventsUserState,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("{:#?}", event);
        Ok(())
    }

    async fn test_command_events_function(
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

    async fn test_other_routes(
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
    info!("Loading server: {}", addr);

    let oauth_listener_config = Arc::new(SlackOAuthListenerConfig::new(
        SlackClientId(config_env_var("SLACK_CLIENT_ID")?),
        SlackClientSecret(config_env_var("SLACK_CLIENT_SECRET")?),
        "users.profile:read".into(),
        "users.profile:read,users.profile:write".into(),
        config_env_var("REDIRECT_HOST")?,
    ));

    debug!(
        "Redirect url: {:#?}",
        oauth_listener_config.to_redirect_url()
    );

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

    info!("Server is listening on {}", &addr);

    loop {
        let (tcp, _) = listener.accept().await?;
        let io = TokioIo::new(tcp);

        let thread_oauth_config = oauth_listener_config.clone();
        let thread_push_events_config = push_events_config.clone();
        let thread_interaction_events_config = interactions_events_config.clone();
        let thread_command_events_config = command_events_config.clone();
        let listener = SlackClientEventsHyperListener::new(listener_environment.clone());
        let routes = chain_service_routes_fn(
            listener.oauth_service_fn(thread_oauth_config, test_oauth_install_function),
            chain_service_routes_fn(
                listener
                    .push_events_service_fn(thread_push_events_config, test_push_events_function),
                chain_service_routes_fn(
                    listener.interaction_events_service_fn(
                        thread_interaction_events_config,
                        test_interaction_events_function,
                    ),
                    chain_service_routes_fn(
                        listener.command_events_service_fn(
                            thread_command_events_config,
                            test_command_events_function,
                        ),
                        test_other_routes,
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
