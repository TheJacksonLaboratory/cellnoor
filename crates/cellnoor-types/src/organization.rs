use nonempty::NonemptyString;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct NewOrganization {
    pub name: NonemptyString,
    pub microsoft_entra_tenant_id: Uuid,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "postgres-types",
    derive(postgres_types::FromSql, postgres_types::ToSql)
)]
#[cfg_attr(feature = "postgres-types", postgres(name = "organization"))]
pub struct Organization {
    pub id: Uuid,
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub name: NonemptyString,
    pub microsoft_entra_tenant_id: Uuid,
}
