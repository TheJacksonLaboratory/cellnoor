use macro_attributes::{base_model, unit_enum};

use crate::specimen::{
    Fixative, SpecimenCommonFields, SpecimenType, SpecimenVariableFields,
    creation::SpecimenInsertion,
};

#[unit_enum]
pub enum BlockFixative {
    FormaldehydeDerivative,
}

impl From<BlockFixative> for Fixative {
    fn from(_: BlockFixative) -> Self {
        Fixative::FormaldehydeDerivative
    }
}

#[base_model]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct NewRnaExtract {
    inner: SpecimenCommonFields,
}

impl NewRnaExtract {
    pub(super) fn common(&self) -> &SpecimenCommonFields {
        &self.inner
    }

    fn into_common(self) -> SpecimenCommonFields {
        self.inner
    }

    pub fn split_for_insertion(self) -> SpecimenInsertion {
        (
            self.into_common(),
            SpecimenVariableFields {
                type_: SpecimenType::RnaExtract,
                embedded_in: None,
                fixative: None,
                thermal_preservation_method: None,
            },
        )
    }
}
