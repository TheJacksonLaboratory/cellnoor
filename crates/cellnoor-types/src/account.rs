use macro_attributes::{base_model, select};
use uuid::Uuid;

use crate::nonempty::NonemptyString;

#[base_model]
#[derive(Copy, strum::AsRefStr)]
#[strum(serialize_all = "snake_case")]
#[cfg_attr(
    feature = "serde",
    serde(deny_unknown_fields, tag = "auth_provider", rename_all = "snake_case")
)]
pub enum NewAccount {
    Microsoft {
        person_id: Uuid,
        microsoft_entra_oid: Uuid,
    },
}

#[select]
#[cfg_attr(feature = "postgres-types", postgres(name = "person_account"))]
pub struct PersonAccount {
    pub id: Uuid,
    pub person_id: Uuid,
    pub name: NonemptyString,
    pub email: Option<NonemptyString>,
    pub auth_provider: String,
    pub auth_provider_user_id: String,
}
