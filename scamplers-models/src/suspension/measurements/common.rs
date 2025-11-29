use macro_attributes::{json, simple_enum};
use macros::{impl_enum_from_sql, impl_enum_to_sql};
use positive::PositiveF32;

#[cfg(feature = "app")]
use crate::utils::{EnumFromSql, EnumToSql};

#[simple_enum]
enum CountingMethod {
    BrightField,
    AcridineOrangePropidiumIodide,
    TrypanBlue,
}

#[cfg(feature = "app")]
impl EnumFromSql for CountingMethod {}
impl_enum_from_sql!(CountingMethod);

#[cfg(feature = "app")]
impl EnumToSql for CountingMethod {}
impl_enum_to_sql!(CountingMethod);

#[simple_enum]
enum Microliter {
    #[serde(alias = "µL")]
    Microliter,
}

#[cfg(feature = "app")]
impl EnumFromSql for Microliter {}
impl_enum_from_sql!(Microliter);

#[cfg(feature = "app")]
impl EnumToSql for Microliter {}
impl_enum_to_sql!(Microliter);

#[simple_enum]
enum Milliliter {
    #[serde(alias = "mL")]
    Milliliter,
}

#[cfg(feature = "app")]
impl EnumFromSql for Milliliter {}
impl_enum_from_sql!(Milliliter);

#[cfg(feature = "app")]
impl EnumToSql for Milliliter {}
impl_enum_to_sql!(Milliliter);

#[simple_enum]
enum Micrometer {
    #[serde(alias = "µm")]
    Micrometer,
}

#[cfg(feature = "app")]
impl EnumFromSql for Micrometer {}
impl_enum_from_sql!(Micrometer);

#[cfg(feature = "app")]
impl EnumToSql for Micrometer {}
impl_enum_to_sql!(Micrometer);

#[simple_enum]
pub enum Cells {
    Cells,
}

#[cfg(feature = "app")]
impl EnumFromSql for Cells {}
impl_enum_from_sql!(Cells);

#[cfg(feature = "app")]
impl EnumToSql for Cells {}
impl_enum_to_sql!(Cells);

#[simple_enum]
pub enum Nuclei {
    Nuclei,
}

#[cfg(feature = "app")]
impl EnumFromSql for Nuclei {}
impl_enum_from_sql!(Nuclei);

#[cfg(feature = "app")]
impl EnumToSql for Nuclei {}
impl_enum_to_sql!(Nuclei);

#[json]
struct Concentration {
    counting_method: Option<CountingMethod>,
    post_hybridization: bool,
    value: PositiveF32,
    denominator_unit: Milliliter,
}

#[json]
struct Viability {
    value: PositiveF32,
}

#[json]
struct Volume {
    post_hybridization: bool,
    value: PositiveF32,
    unit: Microliter,
}

#[json]
struct MeanDiameter {
    post_hybridization: bool,
    value: PositiveF32,
    unit: Micrometer,
}

#[cfg(feature = "app")]
mod rust {
    use super::{Concentration, MeanDiameter, Viability, Volume};
    use jiff::Timestamp;
    use macro_attributes::{insert_select, json};
    use scamplers_schema::suspension_measurements;
    use uuid::Uuid;

    #[json]
    #[serde(tag = "quantity")]
    enum MeasurementData<C> {
        Concentration {
            #[serde(flatten)]
            inner: Concentration,
            numerator_unit: C,
        },
        Viability(Viability),
        Volume(Volume),
        MeanDiameter {
            #[serde(flatten)]
            inner: MeanDiameter,
            object: C,
        },
    }

    #[insert_select]
    #[cfg_attr(feature = "app", diesel(table_name = suspension_measurements))]
    pub struct SuspensionMeasurementFields<C> {
        suspension_id: Uuid,
        measured_by: Uuid,
        #[cfg_attr(feature = "app", diesel(
            serialize_as = jiff_diesel::Timestamp,
            deserialize_as = jiff_diesel::Timestamp
        ))]
        measured_at: Timestamp,
        data: MeasurementData<C>,
    }
}

#[cfg(feature = "app")]
pub use rust::SuspensionMeasurementFields;

#[cfg(all(not(feature = "app"), feature = "typescript"))]
mod typescript {
    use super::{Concentration, MeanDiameter, Viability, Volume};
    use jiff::Timestamp;
    use macro_attributes::{insert_select, json};
    #[cfg(feature = "app")]
    use scamplers_schema::suspension_measurements;
    use ts_rs::TS;
    use uuid::Uuid;

    #[json]
    #[serde(tag = "quantity")]
    enum MeasurementData<C>
    where
        C: TS,
        <C as TS>::OptionInnerType: TS,
    {
        Concentration {
            #[serde(flatten)]
            inner: Concentration,
            numerator_unit: C,
        },
        Viability(Viability),
        Volume(Volume),
        MeanDiameter {
            #[serde(flatten)]
            inner: MeanDiameter,
            object: C,
        },
    }

    #[insert_select]
    struct SuspensionMeasurementFields<C>
    where
        C: TS,
        <C as TS>::OptionInnerType: TS,
    {
        measured_by: Uuid,
        #[cfg_attr(feature = "typescript", ts(as = "Option<String>"))]
        measured_at: Timestamp,
        data: MeasurementData<C>,
    }
}

#[cfg(all(not(feature = "app"), feature = "typescript"))]
pub use typescript::SuspensionMeasurementFields;
