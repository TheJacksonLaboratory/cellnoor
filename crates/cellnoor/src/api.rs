use anyhow::Context;
use axum::{Router, serve::Listener};
pub use routes::router;
use tokio::net::{TcpListener, UnixListener};

use crate::{settings::Settings, state::AppState};

mod routes;

pub async fn serve(settings: &Settings) -> anyhow::Result<()> {
    let app_addr = settings.listen_on().to_owned();

    let app_state = AppState::initialize(&settings).context("failed to initialize app state")?;

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
    println!("cellnoor listening on {:?}", listener.local_addr()?);

    axum::serve(listener, app)
        .await
        .context("failed to serve app")?;

    Ok(())
}

fn api(app_state: AppState) -> Router {
    let (_, api_router) = routes::router();

    // For now, since there's no frontend application, we just serve the app from
    // the root (because we expect that the reverse proxy will serve this app behind
    // api.cellnoor.jax.org)
    Router::new().merge(api_router.with_state(app_state))
}
