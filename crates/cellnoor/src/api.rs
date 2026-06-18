use anyhow::Context;
use axum::{Router, serve::Listener};
use hyper_util::rt::{TokioExecutor, TokioIo};
pub use routes::router;
use tokio::net::{TcpListener, UnixListener};
use tower_http::{normalize_path::NormalizePath, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{settings::Settings, state::AppState};

mod routes;

pub async fn serve(settings: &Settings) -> anyhow::Result<()> {
    let app_addr = settings.listen_on().to_owned();

    let app_state = AppState::initialize(settings).context("failed to initialize app state")?;

    let api = api(app_state);

    initialize_logging();

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

async fn serve_with_listener<L: Listener>(mut listener: L, app: Router) -> anyhow::Result<()>
where
    L::Addr: std::fmt::Debug,
{
    tracing::debug!("cellnoor listening on {:?}", listener.local_addr()?);

    let app = NormalizePath::trim_trailing_slash(app);
    let service = hyper_util::service::TowerToHyperService::new(app);

    loop {
        let (socket, _) = listener.accept().await;
        let service = service.clone();

        tokio::spawn(async move {
            let socket = TokioIo::new(socket);

            // Accept headers of 64 KiB
            if let Err(err) = hyper_util::server::conn::auto::Builder::new(TokioExecutor::new())
                .http2()
                .max_header_list_size(64 * 1024)
                .serve_connection(socket, service)
                .await
            {
                tracing::error!("failed to serve connection: {err}");
            }
        });
    }
}

fn initialize_logging() {
    use tracing_subscriber::{filter::Targets, fmt, registry};

    registry()
        .with(
            Targets::new()
                .with_target("cellnoor", tracing::Level::DEBUG)
                .with_target("axum", tracing::Level::TRACE)
                .with_target("tower_http", tracing::Level::DEBUG),
        )
        .with(fmt::layer())
        .init();
}

fn api(app_state: AppState) -> Router {
    let (_, api_router) = routes::router();

    // For now, since there's no frontend application, we just serve the app from
    // the root (because we expect that the reverse proxy will serve this app behind
    // api.cellnoor.jax.org)
    Router::new()
        .merge(
            api_router
                // Since the file-auth routes use middleware that requires an `AppState`, and we
                // don't want to make the `routes::router` function require an `AppState` as an
                // argument, we just add that route here
                .nest("/file-auth", routes::file_auth::router(app_state.clone()))
                .with_state(app_state),
        )
        .layer(TraceLayer::new_for_http())
}
