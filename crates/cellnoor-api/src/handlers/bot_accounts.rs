pub(super) mod access;
pub(super) mod create;
#[cfg(test)]
mod delete;
pub(super) mod index;
pub(super) mod index_detailed;
pub(super) mod update;

pub use access::{
    add_people::add_people_to_bot_account, remove_people::remove_people_from_bot_account,
};
pub use create::create_bot_account;
pub use index::index_bot_accounts;
pub use index_detailed::index_bot_accounts_detailed;
pub use update::update_bot_account;
