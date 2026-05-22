pub(super) mod create;
pub(super) mod delete;
pub(super) mod index;
pub(super) mod show;
pub(super) mod update;

pub use create::create_institution;
pub use delete::delete_institution;
pub use index::index_institutions;
pub use show::show_institution;
pub use update::update_institution;
