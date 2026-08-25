use macro_attributes::{base_model, discriminant_unit_enum};
use nonempty::NonemptyVec;
use positive::PositiveI32;
use uuid::Uuid;

use crate::{cdna::CdnaSimpleFields, nucleic_acid_measurement::NewNucleicAcidMeasurement};

#[base_model]
pub struct NewCdna {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub simple: CdnaSimpleFields,
    pub gem_well_id: Uuid,
    pub measurements: Vec<NewNucleicAcidMeasurement>,
    pub preparers: NonemptyVec<Uuid>,
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub variable_fields: CdnaVariableFields,
}

#[base_model]
#[derive(Copy, strum::EnumDiscriminants)]
#[cfg_attr(
    feature = "serde",
    serde(tag = "library_type", rename_all = "snake_case")
)]
#[strum(serialize_all = "snake_case")]
#[strum_discriminants(name(LibraryType), discriminant_unit_enum)]
pub enum CdnaVariableFields {
    AntibodyCapture,
    AntigenCapture,
    ChromatinAccessibility,
    CrisprGuideCapture,
    Custom,
    GeneExpression { n_amplification_cycles: PositiveI32 },
    MultiplexingCapture,
    Vdj,
    VdjB,
    VdjT,
    VdjTGd,
}
