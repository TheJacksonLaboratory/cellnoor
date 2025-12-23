use anyhow::Context;
use axum::{Extension, Router, routing::get};
use camino::Utf8Path;
use serde_qs::axum::QsQueryConfig;
use tokio::net::TcpListener;
use zeroize::Zeroize;

use crate::{config::Config, state::AppState};

mod error;
mod extract;
mod routes;

pub use error::{Error, ErrorResponse};

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
        database().await;
    }
    serve_inner(config).await
}

async fn serve_inner(mut config: Config) -> anyhow::Result<()> {
    let app_state = AppState::initialize(&config)
        .await
        .context("failed to initialize app state")?;
    tracing::info!("initialized app state");

    let app = app(app_state.clone());

    let app_addr = config.address();
    let listener = TcpListener::bind(&app_addr)
        .await
        .context(format!("failed to listen on {app_addr}"))?;
    tracing::info!("cellnoor listening on {}", listener.local_addr()?);

    config.zeroize();

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
    // The browser form-encodes everything so we have to enable the less-readable
    // form-encoding
    let query_string_config =
        QsQueryConfig::new().config(serde_qs::Config::new().use_form_encoding(true));
    let api_router = routes::router()
        .route("/health", get(async || "OK"))
        .layer(Extension(query_string_config))
        .with_state(app_state);

    Router::new().nest("/api", api_router)
}
