use macro_attributes::base_model;

use crate::{
    chromium_run::creation::LoadedEntity,
    nonempty::{NonemptyBoundedVec, NonemptyString},
};

pub const MAX_SUSPENSIONS_PER_OCM_GEM_WELL: usize = 4;

// We can't use our normal unit_enum here because it makes everything snake_case
#[derive(Clone, Copy, Debug, PartialEq, Hash, strum::AsRefStr, strum::VariantArray)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "serde", serde(rename_all = "UPPERCASE"))]
#[strum(serialize_all = "UPPERCASE")]
#[cfg_attr(
    feature = "postgres-types",
    derive(postgres_types::ToSql, postgres_types::FromSql),
    postgres(name = "ocm_barcode_id", rename_all = "UPPERCASE")
)]
pub enum OcmBarcodeId {
    Ob1,
    Ob2,
    Ob3,
    Ob4,
}

#[base_model]
#[derive(Copy)]
pub struct OcmLoadedEntity {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub loaded_entity: LoadedEntity,
    pub ocm_barcode_id: OcmBarcodeId,
}

#[base_model]
pub struct NewOcmGemWell {
    pub readable_id: NonemptyString,
    pub loading: NonemptyBoundedVec<OcmLoadedEntity, MAX_SUSPENSIONS_PER_OCM_GEM_WELL>,
}
