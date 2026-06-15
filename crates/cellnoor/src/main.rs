#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use anyhow::Context;
    use cellnoor::{api, settings::Settings};

    dotenvy::dotenv().unwrap_or_default();
    let settings = Settings::read().context("failed to read app configuration")?;

    api::serve(&settings).await?;

    Ok(())
}

// #[cfg(feature = "ssr")]
// async fn leptos_main() {
//     use axum::Router;
//     use camino::Utf8PathBuf;
//     use cellnoor::app::*;
//     use clap::Parser;
//     use leptos::{logging::log, prelude::*};
//     use leptos_axum::{LeptosRoutes, generate_route_list};

//     #[derive(Debug, Parser)]
//     struct Cli {
//         #[clap(short, long)]
//         config_path: Utf8PathBuf,
//     }

//     let Cli { config_path: _ } = Cli::parse();

//     let conf = get_configuration(None).unwrap();
//     let addr = conf.leptos_options.site_addr;
//     let leptos_options = conf.leptos_options;
//     // Generate the list of routes in your Leptos App
//     let routes = generate_route_list(App);

//     let app = Router::new()
//         .leptos_routes(&leptos_options, routes, {
//             let leptos_options = leptos_options.clone();
//             move || shell(leptos_options.clone())
//         })
//         .fallback(leptos_axum::file_and_error_handler(shell))
//         .with_state(leptos_options);

//     log!("listening on http://{}", &addr);
//     let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
//     axum::serve(listener, app.into_make_service())
//         .await
//         .unwrap();
// }
