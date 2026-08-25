#[cfg(feature = "postgres-types")]
use std::str::FromStr;

#[cfg(feature = "postgres-types")]
use bytes::BytesMut;
pub use creation::SpecimenType;
use macro_attributes::{base_model, select, unit_enum};
#[cfg(feature = "postgres-types")]
use postgres_types::{FromSql, ToSql, to_sql_checked};
pub use query::{
    BlockEmbeddingMatrixOperator, FixativeOperator, SimpleSpecimenQuery, SpeciesOperator,
    SpecimenField, SpecimenPredicate, SpecimenQuery, SpecimenTypeOperator,
    ThermalPreservationMethodOperator,
};

use crate::{
    id::{Id, NoId},
    project::{ProjectCompact, SavedProjectRecord},
    simple_links::SimpleLinks,
    specimen::{
        creation::{
            ControlledRateFreezing, DithiobisSuccinimidylpropionate, FlashFreezing,
            FormaldehydeDerivative,
        },
        measurement::SpecimenMeasurement,
        record::SpecimenRecord,
    },
};

pub mod creation;
pub mod measurement;
mod query;

mod record {
    use jiff::Timestamp;
    use macro_attributes::select;
    use nonempty::NonemptyString;
    use uuid::Uuid;

    use crate::specimen::{
        Fixative, Species, SpecimenType, ThermalPreservationMethod,
        creation::block::BlockEmbeddingMatrix,
    };

    #[select]
    #[cfg_attr(feature = "postgres-types", postgres(name = "specimen"))]
    pub struct SpecimenRecord<T> {
        #[cfg_attr(feature = "serde", serde(flatten))]
        pub id: T,
        pub readable_id: NonemptyString,
        pub name: NonemptyString,
        pub submitted_by: Uuid,
        pub project_id: Uuid,
        pub received_at: Timestamp,
        pub species: Species,
        pub host_species: Option<Species>,
        pub returned_at: Option<Timestamp>,
        pub returned_by: Option<Uuid>,
        #[cfg_attr(feature = "postgres-types", postgres(name = "type"))]
        pub type_: SpecimenType,
        pub embedded_in: Option<BlockEmbeddingMatrix>,
        pub fixative: Option<Fixative>,
        pub thermal_preservation_method: Option<ThermalPreservationMethod>,
        pub tissue: NonemptyString,
        pub additional_data: Option<serde_json::Value>,
    }
}

#[unit_enum]
pub enum Species {
    AmbystomaMexicanum,
    CanisFamiliaris,
    CallithrixJacchus,
    DrosophilaMelanogaster,
    GasterosteusAculeatus,
    HomoSapiens,
    MusMusculus,
    RattusNorvegicus,
    SminthopsisCrassicaudata,
}

pub type NewSpecimenRecord = SpecimenRecord<NoId>;

pub type SavedSpecimenRecord = SpecimenRecord<Id>;

#[select]
#[cfg_attr(feature = "postgres-types", postgres(name = "specimen_detailed"))]
pub struct SavedSpecimenRecordDetailed {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub specimen: SavedSpecimenRecord,
    pub project: SavedProjectRecord,
    pub measurements: Vec<SpecimenMeasurement>,
}

#[base_model]
pub struct SpecimenCompact {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: SavedSpecimenRecord,
    pub links: SimpleLinks,
}

// Rather than just wrapping `SavedSpecimenRecordDetailed`, we destructure its
// fields so that we have a `ProjectCompact` rather than a bare
// `SavedProjectRecord`.
#[base_model]
pub struct SpecimenDetailed {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: SavedSpecimenRecord,
    pub links: SimpleLinks,
    pub project: ProjectCompact,
    pub measurements: Vec<SpecimenMeasurement>,
}

#[base_model]
#[derive(Copy)]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum ThermalPreservationMethod {
    ControlledRateFreezing(ControlledRateFreezing),
    FlashFreezing(FlashFreezing),
}

#[cfg(feature = "postgres-types")]
impl<'a> FromSql<'a> for ThermalPreservationMethod {
    fn from_sql(
        ty: &postgres_types::Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let s = <nonempty::NonemptyString as FromSql>::from_sql(ty, raw)?;
        let s = s.as_ref();

        let map_err = |e| Box::new(e) as Box<dyn ::std::error::Error + Sync + Send>;

        let controlled_rate_freezing = ControlledRateFreezing::from_str(s)
            .map(Self::ControlledRateFreezing)
            .map_err(map_err);
        let flash_freezing = FlashFreezing::from_str(s)
            .map(Self::FlashFreezing)
            .map_err(map_err);

        controlled_rate_freezing.or(flash_freezing)
    }

    fn accepts(ty: &postgres_types::Type) -> bool {
        use postgres_types::FromSql;

        <::nonempty::NonemptyString as FromSql>::accepts(ty)
    }
}

#[cfg(feature = "postgres-types")]
impl ToSql for ThermalPreservationMethod {
    to_sql_checked!();

    fn to_sql(
        &self,
        ty: &postgres_types::Type,
        out: &mut BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>>
    where
        Self: Sized,
    {
        let value: &str = match self {
            Self::ControlledRateFreezing(c) => c.as_ref(),
            Self::FlashFreezing(f) => f.as_ref(),
        };

        <&str as ToSql>::to_sql(&value, ty, out)
    }

    fn accepts(ty: &postgres_types::Type) -> bool
    where
        Self: Sized,
    {
        <::nonempty::NonemptyString as ToSql>::accepts(ty) || <&str as ToSql>::accepts(ty)
    }
}

#[base_model]
#[derive(Copy)]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum Fixative {
    DithiobisSuccinimidylpropionate(DithiobisSuccinimidylpropionate),
    FormaldehydeDerivative(FormaldehydeDerivative),
}

#[cfg(feature = "postgres-types")]
impl<'a> FromSql<'a> for Fixative {
    fn from_sql(
        ty: &postgres_types::Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let s = <nonempty::NonemptyString as FromSql>::from_sql(ty, raw)?;
        let s = s.as_ref();

        let map_err = |e| Box::new(e) as Box<dyn ::std::error::Error + Sync + Send>;

        let dsp = DithiobisSuccinimidylpropionate::from_str(s)
            .map(Self::DithiobisSuccinimidylpropionate)
            .map_err(map_err);
        let formaldehyde_derivative = FormaldehydeDerivative::from_str(s)
            .map(Self::FormaldehydeDerivative)
            .map_err(map_err);

        dsp.or(formaldehyde_derivative)
    }

    fn accepts(ty: &postgres_types::Type) -> bool {
        use postgres_types::FromSql;

        <::nonempty::NonemptyString as FromSql>::accepts(ty)
    }
}

#[cfg(feature = "postgres-types")]
impl ToSql for Fixative {
    to_sql_checked!();

    fn to_sql(
        &self,
        ty: &postgres_types::Type,
        out: &mut BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>>
    where
        Self: Sized,
    {
        let value: &str = match self {
            Self::DithiobisSuccinimidylpropionate(c) => c.as_ref(),
            Self::FormaldehydeDerivative(f) => f.as_ref(),
        };

        <&str as ToSql>::to_sql(&value, ty, out)
    }

    fn accepts(ty: &postgres_types::Type) -> bool
    where
        Self: Sized,
    {
        <::nonempty::NonemptyString as ToSql>::accepts(ty) || <&str as ToSql>::accepts(ty)
    }
}
