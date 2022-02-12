use anyhow::Result;
use axum::{
    routing::{get, post},
    AddExtensionLayer, Router,
};
use lettre::{
    transport::smtp::authentication::{Credentials, Mechanism},
    AsyncSmtpTransport, Tokio1Executor,
};
use oso::{Oso, PolarClass};
use sqlx::PgPool;
use std::{str::FromStr, sync::Arc};
use tokio::sync::Mutex;
use tower::ServiceBuilder;
use tower_http::{
    cors::{CorsLayer, Origin},
    trace::TraceLayer,
};

use crate::handlers;

#[derive(Clone)]
pub(crate) struct State {
    pub(crate) oso: Arc<Mutex<Oso>>,
    pub(crate) db_pool: PgPool,
    pub(crate) redis_manager: redis::aio::ConnectionManager,
    pub(crate) mailsender: AsyncSmtpTransport<Tokio1Executor>,
}

impl State {
    /// Attempt to create a new State instance
    pub(crate) async fn try_new() -> Result<State> {
        let oso = Arc::new(Mutex::new(try_register_oso()?));
        let db_pool = PgPool::connect(&std::env::var("DATABASE_URL")?).await?;
        let redis_manager = redis::Client::open(std::env::var("REDIS_URL")?)?
            .get_tokio_connection_manager()
            .await?;
        let mailsender =
            AsyncSmtpTransport::<Tokio1Executor>::relay(&std::env::var("SMTP_SERVER")?)?
                // Add credentials for authentication
                .credentials(Credentials::new(
                    std::env::var("SMTP_USERNAME")?,
                    std::env::var("SMTP_PASSWORD")?,
                ))
                // Configure expected authentication mechanism
                .authentication(vec![Mechanism::Plain])
                .build();

        Ok(State {
            oso,
            db_pool,
            redis_manager,
            mailsender,
        })
    }
}

/// Attempt to create a new oso instance for managing authorization schemes.
fn try_register_oso() -> Result<Oso> {
    use crate::models::*;

    let mut oso = Oso::new();

    // NOTE: load classes here
    oso.register_class(User::get_polar_class())?;

    // NOTE: load oso rule files here
    oso.load_files(vec!["polar/main.polar"])?;

    Ok(oso)
}

/// Attempt to setup the CORS layer.
fn try_cors_layer() -> Result<CorsLayer> {
    use axum::http::Method;

    if cfg!(debug_assertions) {
        Ok(CorsLayer::permissive())
    } else {
        let origins = std::env::var("ALLOWED_ORIGINS")?
            .split(',')
            .map(|s| s.trim().parse())
            .collect::<Result<Vec<_>, _>>()?;

        Ok(CorsLayer::new()
            // allow `GET`, `POST`, `PUT`, `DELETE`
            .allow_methods(vec![Method::GET, Method::POST, Method::PUT, Method::DELETE])
            // allow credentials
            .allow_credentials(true)
            // allow requests from specified env origins
            .allow_origin(Origin::list(origins)))
    }
}

/// Run the server.
pub(crate) async fn run() -> Result<()> {
    let addr = std::net::SocketAddr::from_str(&std::env::var("ADDR")?)?;
    tracing::debug!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(try_app().await?.into_make_service())
        .await?;
    Ok(())
}

async fn try_app() -> Result<Router> {
    let state = State::try_new().await?;

    let middleware_stack = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(AddExtensionLayer::new(state))
        .layer(try_cors_layer()?);

    Ok(Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/user", post(handlers::create_user))
        .route(
            "/user/verify",
            post(handlers::create_verify_user).put(handlers::update_verify_user),
        )
        .route(
            "/user/:id",
            get(handlers::get_user).put(handlers::update_user),
        )
        .route("/login", post(handlers::login).delete(handlers::logout))
        .layer(middleware_stack))
}
