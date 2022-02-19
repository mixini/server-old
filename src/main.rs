#![warn(
    missing_debug_implementations,
    unreachable_pub,
    future_incompatible,
    rust_2018_idioms,
    rust_2021_compatibility
)]

pub mod actions;
pub mod auth;
pub mod constants;
pub mod error;
pub mod handlers;
pub mod server;
pub mod utils;

pub const DEV_BUILD: bool = cfg!(debug_assertions);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if DEV_BUILD {
        dotenv::dotenv().ok();
    } else {
        dotenv::from_filename("prod.env").ok();
    }

    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var(
            "RUST_LOG",
            "mixini_server=debug,tower_http=debug,sea_orm=debug",
        )
    }
    tracing_subscriber::fmt::init();
    server::run().await
}
