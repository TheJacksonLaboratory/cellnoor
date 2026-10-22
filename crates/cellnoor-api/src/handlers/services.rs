pub(super) mod access;
pub(super) mod create;
#[cfg(test)]
mod delete;
pub(super) mod index;
pub(super) mod index_detailed;
pub(super) mod update;

pub use access::{add_people::add_people_to_service, remove_people::remove_people_from_service};
pub use create::create_service;
pub use index::index_services;
pub use index_detailed::index_services_detailed;
pub use update::update_service;
