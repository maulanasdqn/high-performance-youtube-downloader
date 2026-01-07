mod api_doc;
mod config;
mod health;
mod logger;
mod router;
mod use_cases;

use config::{load_environment_config, Config};
use logger::init_logger;
use router::build_router;
use use_cases::init_use_cases;

fn setup_app(config: &Config) -> axum::Router {
    let use_cases = init_use_cases(config);
    build_router(use_cases, config)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logger()?;

    let config = load_environment_config();

    std::fs::create_dir_all(&config.download_dir)?;
    tracing::info!(download_dir = %config.download_dir, "Download directory ready");

    let app = setup_app(&config);

    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("Server listening on {}", listener.local_addr()?);
    tracing::info!("API Documentation available at http://{addr}/docs");

    axum::serve(listener, app).await?;

    Ok(())
}
