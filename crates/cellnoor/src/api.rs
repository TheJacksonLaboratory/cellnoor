use anyhow::Context;
use axum::{Router, serve::Listener};
use camino::Utf8Path;
use tokio::net::{TcpListener, UnixListener};

use crate::{settings::Settings, state::AppState};

mod routes;

pub async fn serve(config_path: Option<&Utf8Path>) -> anyhow::Result<()> {
    let settings = Settings::read(config_path).context("failed to read app settings")?;

    let app_addr = settings.address().to_owned();

    let app_state = AppState::initialize(settings)
        .await
        .context("failed to initialize app state")?;

    let api = api(app_state);

    if app_addr.starts_with('/') {
        if let Err(e) = std::fs::remove_file(&app_addr)
            && !matches!(e.kind(), std::io::ErrorKind::NotFound)
        {
            return Err(e)?;
        }

        let listener =
            UnixListener::bind(&app_addr).context(format!("failed to listen on {app_addr}"))?;
        serve_with_listener(listener, api).await
    } else {
        let listener = TcpListener::bind(&app_addr)
            .await
            .context(format!("failed to listen on {app_addr}"))?;
        serve_with_listener(listener, api).await
    }
}

async fn serve_with_listener<L: Listener>(listener: L, app: Router) -> anyhow::Result<()>
where
    L::Addr: std::fmt::Debug,
{
    leptos::logging::log!("cellnoor listening on {:?}", listener.local_addr()?);

    axum::serve(listener, app)
        .await
        .context("failed to serve app")?;

    Ok(())
}

fn api(app_state: AppState) -> Router {
    let api_router = routes::app(app_state.clone()).with_state(app_state);

    Router::new().nest("/api", api_router)
}
