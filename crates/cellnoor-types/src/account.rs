use macro_attributes::select;
use uuid::Uuid;

use crate::nonempty::NonemptyString;

// This is a read-only model because it's written by better-auth
#[select]
#[cfg_attr(feature = "postgres-types", postgres(name = "person_account"))]
pub struct PersonAccount {
    pub id: Uuid,
    pub name: NonemptyString,
    pub email: Option<NonemptyString>,
    pub auth_provider: String,
    pub auth_provider_user_id: String,
}
