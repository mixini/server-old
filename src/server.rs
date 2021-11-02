use anyhow::Result;
use axum::{
    extract::Extension,
    handler::{get, post},
    routing::BoxRoute,
    AddExtensionLayer, Router,
};
use lettre::{
    transport::smtp::authentication::{Credentials, Mechanism},
    SmtpTransport,
};
use oso::Oso;
use sqlx::PgPool;
use std::{str::FromStr, sync::Arc};
use tokio::sync::Mutex;
use tower_http::trace::TraceLayer;

use crate::handlers;

#[derive(Clone)]
pub(crate) struct State {
    pub(crate) oso: Arc<Mutex<Oso>>,
    pub(crate) db_pool: PgPool,
    pub(crate) redis_manager: redis::aio::ConnectionManager,
    pub(crate) mailer: SmtpTransport,
}

impl State {
    /// Attempt to create a new State instance
    pub(crate) async fn try_new() -> Result<State> {
        let oso = Arc::new(Mutex::new(try_register_oso()?));
        let db_pool = PgPool::connect(&std::env::var("DATABASE_URL")?).await?;
        let redis_manager = redis::Client::open(std::env::var("REDIS_URL")?)?
            .get_tokio_connection_manager()
            .await?;
        let mailer = SmtpTransport::starttls_relay(&std::env::var("SMTP_SERVER")?)?
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
            mailer,
        })
    }
}

/// Attempt to create a new oso instance for managing authorization schemes.
pub(crate) fn try_register_oso() -> Result<Oso> {
    let mut oso = Oso::new();

    // NOTE: load oso rule files here
    oso.load_files(vec!["polar/base.polar"])?;

    Ok(oso)
}

/// Run the server.
pub(crate) async fn run() -> Result<()> {
    // // endpoints
    // app.at("/").get(|_| async { Ok("Hello, world!") });
    // app.at("/register").post(endpoints::register);
    // app.at("/register/verify").put(endpoints::register_verify);

    let addr = std::net::SocketAddr::from_str(&std::env::var("ADDR")?)?;
    tracing::debug!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(try_app().await?.into_make_service())
        .await?;
    Ok(())
}

async fn try_app() -> Result<Router<BoxRoute>> {
    let state = State::try_new().await?;

    Ok(Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/user", post(handlers::new_user))
        .layer(TraceLayer::new_for_http())
        .layer(AddExtensionLayer::new(state))
        .boxed())
}
