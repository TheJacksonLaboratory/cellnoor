use jiff::Timestamp;
use macro_attributes::{base_model, discriminant_unit_enum};
use nonempty::{NonemptyString, NonemptyVec};
use positive::PositiveI32;
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
#[strum_discriminants(name(LibraryType), discriminant_unit_enum)]
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
