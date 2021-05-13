use anyhow::Result;
use oso::Oso;
use std::sync::Arc;
use tide::log::{self, LogMiddleware};
use tokio::sync::Mutex;

use sqlx::PgPool;

use crate::endpoints;

#[derive(Clone)]
pub(crate) struct State {
    pub(crate) oso: Arc<Mutex<Oso>>,
    pub(crate) db_pool: PgPool,
    pub(crate) redis_client: redis::Client,
}

impl State {
    /// Attempt to create a new State instance
    pub(crate) async fn try_new() -> Result<State> {
        let oso = Arc::new(Mutex::new(try_register_oso()?));
        let db_pool = PgPool::connect(&std::env::var("DATABASE_URL")?).await?;
        let redis_client = redis::Client::open(std::env::var("REDIS_URL")?)?;

        Ok(State {
            oso,
            db_pool,
            redis_client,
        })
    }
}

/// Attempt to create a new oso instance for managing authorization schemes.
pub(crate) fn try_register_oso() -> Result<Oso> {
    let oso = Oso::new();

    oso.load_file("polar/base.polar")?;

    Ok(oso)
}

/// Run the server.
pub(crate) async fn run() -> Result<()> {
    log::start();

    let mut app = tide::with_state(State::try_new().await?);

    // middlewares
    app.with(LogMiddleware::new());

    // endpoints
    app.at("/").get(|_| async { Ok("Hello, world!") });
    app.at("/register").post(endpoints::register);

    app.listen(std::env::var("ADDR")?).await?;
    Ok(())
}
