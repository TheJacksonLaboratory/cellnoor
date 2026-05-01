#[cfg(feature = "ssr")]
pub mod api;
pub mod app;
#[cfg(feature = "ssr")]
pub mod auth;
#[cfg(feature = "ssr")]
pub mod db;
#[cfg(feature = "ssr")]
pub mod error;
#[cfg(feature = "ssr")]
pub mod settings;
#[cfg(feature = "ssr")]
pub mod state;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
