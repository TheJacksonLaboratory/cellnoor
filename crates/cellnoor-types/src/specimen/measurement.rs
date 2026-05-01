use jiff::Timestamp;
use macro_attributes::base_model;
use nonempty::NonemptyString;
use positive::PositiveF32;
use uuid::Uuid;

#[base_model]
pub struct NewSpecimenMeasurement {
    pub measured_by: Uuid,
    pub measured_at: Timestamp,
    pub data: SpecimenMeasurementData,
}

#[base_model]
pub struct SpecimenMeasurement {
    pub id: Uuid,
    pub specimen_id: Uuid,
    pub measured_by: Uuid,
    pub measured_at: Timestamp,
    pub data: SpecimenMeasurementData,
}

#[base_model]
#[cfg_attr(feature = "serde", serde(tag = "quantity"))]
pub enum SpecimenMeasurementData {
    #[cfg_attr(feature = "serde", serde(rename = "DV200"))]
    Dv200 {
        instrument_name: Option<NonemptyString>,
        value: PositiveF32,
    },
    #[cfg_attr(feature = "serde", serde(rename = "RIN"))]
    Rin {
        instrument_name: Option<NonemptyString>,
        value: PositiveF32,
    },
}
