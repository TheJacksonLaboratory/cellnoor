use anyhow::Context;
#[cfg(any(feature = "dummy-data", test))]
pub use auth::AuthProjects;
use axum::{Router, serve::Listener};
use camino::Utf8Path;
use tokio::net::{TcpListener, UnixListener};

use crate::{config::Config, state::AppState};

mod auth;
pub mod error;
mod extract;
pub mod middleware;
pub mod routes;
pub mod util;

#[cfg(test)]
pub async fn serve_integration_test(config: Config) -> anyhow::Result<()> {
    serve_inner(config).await
}

pub async fn serve(config: Config) -> anyhow::Result<()> {
    #[cfg(feature = "dummy-data")]
    use crate::test_state::database;

    initialize_logging(config.log_dir());
    #[cfg(feature = "dummy-data")]
    {
        // This populates the database with dummy-data
        #[allow(clippy::large_futures)]
        database().await;
    }
    serve_inner(config).await
}

async fn serve_inner(config: Config) -> anyhow::Result<()> {
    let app_addr = config.address().to_owned();

    let app_state = AppState::initialize(config)
        .await
        .context("failed to initialize app state")?;
    tracing::info!("initialized app state");

    let app = app(app_state.clone());

    if app_addr.starts_with('/') {
        if let Err(e) = std::fs::remove_file(&app_addr)
            && !matches!(e.kind(), std::io::ErrorKind::NotFound)
        {
            return Err(e)?;
        }

        let listener =
            UnixListener::bind(&app_addr).context(format!("failed to listen on {app_addr}"))?;
        serve_with_listener(listener, app).await
    } else {
        let listener = TcpListener::bind(&app_addr)
            .await
            .context(format!("failed to listen on {app_addr}"))?;
        serve_with_listener(listener, app).await
    }
}

async fn serve_with_listener<L: Listener>(listener: L, app: Router) -> anyhow::Result<()>
where
    L::Addr: std::fmt::Debug,
{
    tracing::info!("cellnoor listening on {:?}", listener.local_addr()?);

    axum::serve(listener, app)
        .await
        .context("failed to serve app")?;

    Ok(())
}

fn initialize_logging(log_dir: Option<&Utf8Path>) {
    use tracing::Level;
    use tracing_subscriber::{filter::Targets, prelude::*};

    let log_layer = tracing_subscriber::fmt::layer();

    match log_dir {
        None => {
            let dev_test_log_filter = Targets::new().with_target("cellnoor", Level::DEBUG);
            let log_layer = log_layer.pretty().with_filter(dev_test_log_filter);

            tracing_subscriber::registry().with(log_layer).init();
        }
        Some(path) => {
            let log_writer = tracing_appender::rolling::daily(path, "cellnoor.log");
            let prod_log_filter = Targets::new().with_target("cellnoor", Level::INFO);
            let log_layer = log_layer
                .json()
                .with_writer(log_writer)
                .with_filter(prod_log_filter);

            tracing_subscriber::registry().with(log_layer).init();
        }
    }
}

fn app(app_state: AppState) -> Router {
    let api_router = routes::app(app_state.clone()).with_state(app_state);

    Router::new().nest("/api", api_router)
}
