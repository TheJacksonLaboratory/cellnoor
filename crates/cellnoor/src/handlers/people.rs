pub(super) mod create;
pub(super) mod delete;
pub(super) mod index;
pub(super) mod show;
pub(super) mod update;

pub use create::create_person;
pub use delete::delete_person;
pub use index::index_people;
pub use show::show_person;
pub use update::update_person;
