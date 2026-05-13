use macro_attributes::base_model;

use crate::specimen::{
    Fixative, SpecimenType, ThermalPreservationMethod,
    creation::{NewSpecimenCommonFields, SpecimenInsertion},
};

#[base_model]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct NewRnaExtract {
    inner: NewSpecimenCommonFields,
}

impl NewRnaExtract {
    pub(crate) fn split_for_insertion(self) -> SpecimenInsertion {
        SpecimenInsertion::from_fields(
            self.inner,
            SpecimenType::RnaExtract,
            None,
            None::<Fixative>,
            None::<ThermalPreservationMethod>,
        )
    }
}
