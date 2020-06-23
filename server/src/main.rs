use tide::log::LogMiddleware;

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    let mut app = tide::new();

    let log_middleware = LogMiddleware::new();
    app.middleware(log_middleware);

    app.at("/static").serve_dir("static/")?;

    app.listen("127.0.0.1:3030").await?;
    Ok(())
}
