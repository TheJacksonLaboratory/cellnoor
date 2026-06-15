pub(super) mod access;
pub(super) mod create;
pub(super) mod delete;
pub(super) mod index_compact;
pub(super) mod index_detailed;
pub(super) mod show;
pub(super) mod update;

pub use access::add_people::add_people_to_project;
pub use create::create_project;
pub use delete::delete_project;
pub use index_compact::index_projects;
pub use index_detailed::index_projects_detailed;
pub use show::show_project;
pub use update::update_project;
