#![warn(
    missing_debug_implementations,
    unreachable_pub,
    future_incompatible,
    rust_2018_idioms
)]

pub(crate) mod endpoints;
pub(crate) mod middleware;
pub(crate) mod models;
pub(crate) mod server;

#[tokio::main]
async fn main() -> Result<(), anyhow::Result<()>> {
    dotenv::dotenv().ok();
    if let Err(err) = server::run().await {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
    Ok(())
}
