use macro_attributes::select;
use nonempty::NonemptyString;
use uuid::Uuid;

#[select]
#[cfg_attr(feature = "postgres-types", postgres(name = "institution"))]
#[cfg_attr(feature = "schemars", schemars(inline))]
pub struct InstitutionRecord<T> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub id: T,
    pub name: NonemptyString,
    pub microsoft_entra_tenant_id: Uuid,
}
