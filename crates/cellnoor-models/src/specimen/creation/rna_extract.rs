use macro_attributes::{base_model, simple_enum};

use crate::specimen::{
    common::SpecimenCommonFields,
    creation::SpecimenInsertion,
    variable::{Fixative, SpecimenType, SpecimenVariableFields},
};

#[simple_enum]
#[derive(strum::VariantArray)]
pub enum BlockFixative {
    FormaldehydeDerivative,
}

impl From<BlockFixative> for Fixative {
    fn from(_: BlockFixative) -> Self {
        Fixative::FormaldehydeDerivative
    }
}

#[base_model]
#[derive(serde::Deserialize)]
#[cfg_attr(feature = "app", derive(schemars::JsonSchema))]
pub struct NewRnaExtract {
    #[serde(flatten)]
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
