use jiff::Timestamp;
use macro_attributes::{base_model, sort_field_enum};
use nonempty::{NonemptyString, NonemptyVec};
use positive::PositiveI32;
#[cfg(feature = "postgres-types")]
use postgres_types::{FromSql, ToSql, to_sql_checked};
use uuid::Uuid;

use crate::nucleic_acid_measurement::NewNucleicAcidMeasurement;

#[base_model]
pub struct CdnaSimpleFields {
    pub readable_id: NonemptyString,
    pub prepared_at: Timestamp,
    pub additional_data: Option<serde_json::Value>,
}

#[base_model]
pub struct NewCdnaCommonFields {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: CdnaSimpleFields,
    pub measurements: Vec<NewNucleicAcidMeasurement>,
    pub preparers: NonemptyVec<Uuid>,
}

#[base_model]
pub struct NewChromiumCdnaCommonFields {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub common: NewCdnaCommonFields,
    pub gem_well_id: Uuid,
}

#[base_model]
#[cfg_attr(
    feature = "serde",
    serde(tag = "library_type", rename_all = "snake_case")
)]
#[derive(strum::EnumDiscriminants, strum::AsRefStr)]
#[strum(serialize_all = "snake_case")]
#[strum_discriminants(name(LibraryType), sort_field_enum)]
pub enum NewCdna {
    AntibodyCapture(NewChromiumCdnaCommonFields),
    AntigenCapture(NewChromiumCdnaCommonFields),
    ChromatinAccessibility(NewChromiumCdnaCommonFields),
    CrisprGuideCapture(NewChromiumCdnaCommonFields),
    Custom(NewChromiumCdnaCommonFields),
    GeneExpression {
        #[cfg_attr(feature = "serde", serde(flatten))]
        common: NewChromiumCdnaCommonFields,
        n_amplification_cycles: PositiveI32,
    },
    MultiplexingCapture(NewChromiumCdnaCommonFields),
    Vdj(NewChromiumCdnaCommonFields),
    VdjB(NewChromiumCdnaCommonFields),
    VdjT(NewChromiumCdnaCommonFields),
    VdjTGd(NewChromiumCdnaCommonFields),
}

impl NewCdna {
    pub fn gem_well_id(&self) -> Option<Uuid> {
        use NewCdna::*;

        match self {
            AntibodyCapture(common)
            | AntigenCapture(common)
            | ChromatinAccessibility(common)
            | CrisprGuideCapture(common)
            | Custom(common)
            | GeneExpression { common, .. }
            | MultiplexingCapture(common)
            | Vdj(common)
            | VdjB(common)
            | VdjT(common)
            | VdjTGd(common) => Some(common.gem_well_id),
        }
    }
}

#[cfg(feature = "postgres-types")]
impl<'a> FromSql<'a> for LibraryType {
    fn from_sql(
        ty: &postgres_types::Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        use std::str::FromStr;

        NonemptyString::from_sql(ty, raw).map(|s| Self::from_str(s.as_ref()).unwrap())
    }

    fn accepts(ty: &postgres_types::Type) -> bool {
        <NonemptyString as FromSql>::accepts(ty)
    }
}

#[cfg(feature = "postgres-types")]
impl ToSql for LibraryType {
    to_sql_checked!();

    fn to_sql(
        &self,
        ty: &postgres_types::Type,
        out: &mut bytes::BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>>
    where
        Self: Sized,
    {
        let as_str: &str = self.into();

        NonemptyString::new(as_str.to_owned())
            .unwrap()
            .to_sql(ty, out)
    }

    fn accepts(ty: &postgres_types::Type) -> bool
    where
        Self: Sized,
    {
        <NonemptyString as ToSql>::accepts(ty)
    }
}
