#![warn(
    missing_debug_implementations,
    unreachable_pub,
    future_incompatible,
    rust_2018_idioms,
    rust_2021_compatibility
)]

pub(crate) mod actions;
pub(crate) mod auth;
pub(crate) mod constants;
pub(crate) mod error;
pub(crate) mod handlers;
pub(crate) mod server;
pub(crate) mod utils;

#[tokio::main]
async fn main() -> Result<(), anyhow::Result<()>> {
    if cfg!(debug_assertions) {
        dotenv::dotenv().ok();
    } else {
        dotenv::from_filename("prod.env").ok();
    }

    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "mixini_server=debug,tower_http=debug")
    }
    tracing_subscriber::fmt::init();

    if let Err(err) = server::run().await {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
    Ok(())
}
