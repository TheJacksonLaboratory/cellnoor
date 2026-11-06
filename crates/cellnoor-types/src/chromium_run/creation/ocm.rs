use macro_attributes::base_model;

use crate::{
    chromium_run::creation::LoadedEntity,
    nonempty::{NonemptyBoundedVec, NonemptyString},
};

pub const MAX_SUSPENSIONS_PER_OCM_GEM_WELL: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, strum::AsRefStr, strum::EnumString, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "serde", serde(rename_all = "UPPERCASE"))]
#[strum(serialize_all = "UPPERCASE")]
pub enum OcmBarcodeId {
    Ob1,
    Ob2,
    Ob3,
    Ob4,
}

#[cfg(feature = "postgres-types")]
mod sql_impls {
    use ::bytes::BytesMut;
    use ::postgres_types::{FromSql, ToSql, to_sql_checked};
    use ::std::str::FromStr;

    use super::OcmBarcodeId;

    impl<'a> FromSql<'a> for OcmBarcodeId {
        fn from_sql(
            ty: &postgres_types::Type,
            raw: &'a [u8],
        ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
            let value = <crate::nonempty::NonemptyString as FromSql>::from_sql(ty, raw)?;

            Ok(Self::from_str(value.as_ref())?)
        }

        fn accepts(ty: &postgres_types::Type) -> bool {
            <crate::nonempty::NonemptyString as FromSql>::accepts(ty)
        }
    }

    impl ToSql for OcmBarcodeId {
        to_sql_checked!();

        fn to_sql(
            &self,
            ty: &postgres_types::Type,
            out: &mut BytesMut,
        ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>>
        where
            Self: Sized,
        {
            let value: &str = self.as_ref();
            <&str as ToSql>::to_sql(&value, ty, out)
        }

        fn accepts(ty: &postgres_types::Type) -> bool
        where
            Self: Sized,
        {
            <crate::nonempty::NonemptyString as ToSql>::accepts(ty) || <&str as ToSql>::accepts(ty)
        }
    }
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
