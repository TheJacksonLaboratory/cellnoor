use macro_attributes::{base_model, discriminant_unit_enum};
use uuid::Uuid;

use crate::{
    cdna::CdnaSimpleFields, nonempty::NonemptyVec,
    nucleic_acid_measurement::NewNucleicAcidMeasurement, positive::PositiveI32,
};

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
#[cfg_attr(feature = "serde", serde(tag = "library_type"))]
#[strum_discriminants(name(LibraryType), discriminant_unit_enum)]
pub enum CdnaVariableFields {
    #[cfg_attr(feature = "serde", serde(rename = "Antibody Capture"))]
    #[cfg_attr(feature = "serde", strum_discriminants(serde(rename = "Antibody Capture")))]
    #[strum_discriminants(strum(serialize = "Antibody Capture"))]
    AntibodyCapture,
    #[cfg_attr(feature = "serde", serde(rename = "Antigen Capture"))]
    #[cfg_attr(feature = "serde", strum_discriminants(serde(rename = "Antigen Capture")))]
    #[strum_discriminants(strum(serialize = "Antigen Capture"))]
    AntigenCapture,
    #[cfg_attr(feature = "serde", serde(rename = "Chromatin Accessibility"))]
    #[cfg_attr(feature = "serde", strum_discriminants(serde(rename = "Chromatin Accessibility")))]
    #[strum_discriminants(strum(serialize = "Chromatin Accessibility"))]
    ChromatinAccessibility,
    #[cfg_attr(feature = "serde", serde(rename = "CRISPR Guide Capture"))]
    #[cfg_attr(feature = "serde", strum_discriminants(serde(rename = "CRISPR Guide Capture")))]
    #[strum_discriminants(strum(serialize = "CRISPR Guide Capture"))]
    CrisprGuideCapture,
    #[cfg_attr(feature = "serde", serde(rename = "Custom"))]
    #[cfg_attr(feature = "serde", strum_discriminants(serde(rename = "Custom")))]
    #[strum_discriminants(strum(serialize = "Custom"))]
    Custom,
    #[cfg_attr(feature = "serde", serde(rename = "Gene Expression"))]
    #[cfg_attr(feature = "serde", strum_discriminants(serde(rename = "Gene Expression")))]
    #[strum_discriminants(strum(serialize = "Gene Expression"))]
    GeneExpression {
        n_amplification_cycles: PositiveI32,
    },
    #[cfg_attr(feature = "serde", serde(rename = "Multiplexing Capture"))]
    #[cfg_attr(feature = "serde", strum_discriminants(serde(rename = "Multiplexing Capture")))]
    #[strum_discriminants(strum(serialize = "Multiplexing Capture"))]
    MultiplexingCapture,
    #[cfg_attr(feature = "serde", serde(rename = "VDJ"))]
    #[cfg_attr(feature = "serde", strum_discriminants(serde(rename = "VDJ")))]
    #[strum_discriminants(strum(serialize = "VDJ"))]
    Vdj,
    #[cfg_attr(feature = "serde", serde(rename = "VDJ B"))]
    #[cfg_attr(feature = "serde", strum_discriminants(serde(rename = "VDJ B")))]
    #[strum_discriminants(strum(serialize = "VDJ B"))]
    VdjB,
    #[cfg_attr(feature = "serde", serde(rename = "VDJ T"))]
    #[cfg_attr(feature = "serde", strum_discriminants(serde(rename = "VDJ T")))]
    #[strum_discriminants(strum(serialize = "VDJ T"))]
    VdjT,
    #[cfg_attr(feature = "serde", serde(rename = "VDJ T GD"))]
    #[cfg_attr(feature = "serde", strum_discriminants(serde(rename = "VDJ T GD")))]
    #[strum_discriminants(strum(serialize = "VDJ T GD"))]
    VdjTGd,
}
