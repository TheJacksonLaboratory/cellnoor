pub(super) mod create;
pub(super) mod delete;
pub(super) mod index_compact;
pub(super) mod index_detailed;
pub(super) mod measurements;
pub(super) mod show;
pub(super) mod update;

pub use create::create_specimen;
pub use delete::delete_specimen;
pub use index_compact::index_specimens;
pub use index_detailed::index_specimens_detailed;
pub use measurements::create::create_specimen_measurement;
pub use show::show_specimen;
pub use update::update_specimen;
