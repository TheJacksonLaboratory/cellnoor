use jiff::Timestamp;
use macro_attributes::{base_model, select};
pub use query::{BotAccountField, BotAccountPredicate, BotAccountQuery, SimpleBotAccountQuery};
use uuid::Uuid;

use crate::{Relation, nonempty::NonemptyString, permission::Permission};

mod query;

#[base_model]
pub struct BotAccountSimpleFields {
    pub description: Option<NonemptyString>,
    pub is_staff: bool,
}

#[base_model]
pub struct NewBotAccount {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: BotAccountSimpleFields,
    pub users: Vec<Uuid>,
    pub permissions_to_grant: Vec<Permission>,
}

pub type BotAccountUpdate = BotAccountSimpleFields;

impl Relation for BotAccountSimpleFields {
    const NAME: &'static str = "bot_account";
}

#[select]
#[cfg_attr(feature = "postgres-types", postgres(name = "bot_account_public"))]
pub struct BotAccount {
    pub id: Uuid,
    pub description: Option<NonemptyString>,
    pub owned_by: Uuid,
    pub is_staff: bool,
    pub created_at: Timestamp,
}

impl Relation for BotAccount {
    const NAME: &'static str = "bot_account_public";
}

#[select]
#[cfg_attr(feature = "postgres-types", postgres(name = "bot_account_detailed"))]
pub struct BotAccountDetailed {
    #[cfg_attr(feature = "serde", serde(flatten))]
    #[cfg_attr(feature = "postgres-types", postgres(name = "bot_account_public"))]
    pub bot_account: BotAccount,
    pub permissions: Vec<Permission>,
    pub members: Vec<Uuid>,
}
