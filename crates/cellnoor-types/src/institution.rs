use macro_attributes::base_model;
pub use query::{InstitutionField, InstitutionPredicate, InstitutionQuery, SimpleInstitutionQuery};

use crate::{
    id::{Id, NoId},
    institution::record::InstitutionRecord,
    simple_links::SimpleLinks,
};

mod query;

mod record {
    use macro_attributes::select;
    use nonempty::NonemptyString;
    use uuid::Uuid;

    #[select]
    #[cfg_attr(feature = "postgres-types", postgres(name = "institution"))]
    // We name it `NewInstitution` so it shows up as `NewInstitution` in the schema and because it
    // just gets flattened into the definition of `Institution`, so the type is only named once
    #[cfg_attr(feature = "schemars", schemars(rename = "NewInstitution"))]
    pub struct InstitutionRecord<T> {
        #[cfg_attr(feature = "serde", serde(flatten))]
        pub id: T,
        pub name: NonemptyString,
        pub microsoft_entra_tenant_id: Uuid,
    }
}

pub type NewInstitution = InstitutionRecord<NoId>;

pub type SavedInstitutionRecord = InstitutionRecord<Id>;

#[base_model]
#[cfg_attr(feature = "schemars", schemars(rename = "Institution"))]
pub struct Institution {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: SavedInstitutionRecord,
    pub links: SimpleLinks,
}
