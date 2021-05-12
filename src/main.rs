mod endpoints;
mod middleware;
mod models;
mod server;

#[tokio::main]
async fn main() -> Result<(), anyhow::Result<()>> {
    dotenv::dotenv().ok();
    if let Err(err) = server::run().await {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
    Ok(())
}
