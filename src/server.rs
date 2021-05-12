use async_std::sync::{Arc, Mutex};
use oso::{Oso, OsoError};
use tide::log::{self, LogMiddleware};

#[derive(Clone)]
pub struct State {
    pub oso: Arc<Mutex<Oso>>,
}

impl State {
    /// Attempt to create a new State instance
    pub fn try_new() -> Result<State, OsoError> {
        let oso = Arc::new(Mutex::new(try_register_oso()?));

        Ok(State { oso })
    }
}

fn try_register_oso() -> Result<Oso, OsoError> {
    let oso = Oso::new();

    // TODO: register polar files

    Ok(oso)
}

pub async fn run() -> Result<(), tide::Error> {
    log::start();

    let mut app = tide::with_state(State::try_new()?);

    // middlewares
    app.with(LogMiddleware::new());

    // endpoints
    app.at("/").get(|_| async { Ok("Hello, world!") });

    app.listen(std::env::var("ADDR")?).await?;
    Ok(())
}
