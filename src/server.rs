use anyhow::Result;
use axum::{
    routing::{get, post},
    AddExtensionLayer, Router,
};
use lettre::{
    transport::smtp::authentication::{Credentials, Mechanism},
    AsyncSmtpTransport, Tokio1Executor,
};
use oso::Oso;
use sea_orm::{Database, DatabaseConnection};
use std::{str::FromStr, sync::Arc};
use tokio::sync::Mutex;
use tower::ServiceBuilder;
use tower_http::{
    cors::{CorsLayer, Origin},
    trace::TraceLayer,
};

use crate::{actions::try_register_oso, handlers};

#[derive(Clone)]
pub struct State {
    pub oso: Arc<Mutex<Oso>>,
    pub db: DatabaseConnection,
    pub redis_manager: redis::aio::ConnectionManager,
    pub mailsender: AsyncSmtpTransport<Tokio1Executor>,
}

impl State {
    /// Attempt to create a new State instance
    pub async fn try_new() -> Result<State> {
        let oso = Arc::new(Mutex::new(try_register_oso()?));
        let db = Database::connect(&std::env::var("DATABASE_URL")?).await?;
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
            db,
            redis_manager,
            mailsender,
        })
    }
}

/// Attempt to setup the CORS layer.
fn try_cors_layer() -> Result<CorsLayer> {
    use axum::http::Method;

    if crate::DEV_BUILD {
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
pub async fn run() -> Result<()> {
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
            get(handlers::get_user)
                .put(handlers::update_user)
                .delete(handlers::delete_user),
        )
        .route("/login", post(handlers::login).delete(handlers::logout))
        .layer(middleware_stack))
}
