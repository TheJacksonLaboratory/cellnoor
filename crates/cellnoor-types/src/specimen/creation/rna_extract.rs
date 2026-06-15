use macro_attributes::base_model;

use crate::specimen::{
    Fixative, SpecimenType, ThermalPreservationMethod,
    creation::{NewSpecimenCommonFields, SpecimenInsertion},
};

#[base_model]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct NewRnaExtract {
    pub common: NewSpecimenCommonFields,
}

impl NewRnaExtract {
    pub(super) fn split_for_insertion(self) -> SpecimenInsertion {
        SpecimenInsertion::from_fields(
            self.common,
            SpecimenType::RnaExtract,
            None,
            None::<Fixative>,
            None::<ThermalPreservationMethod>,
        )
    }
}
