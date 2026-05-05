use macro_attributes::select;
use uuid::Uuid;

use crate::specimen::{SpecimenCommonFields, SpecimenVariableFields};

#[select]
#[cfg_attr(feature = "postgres-types", postgres(name = "specimen"))]
pub struct SpecimenRecord {
    id: Uuid,
    #[cfg_attr(feature = "serde", serde(flatten))]
    common: SpecimenCommonFields,
    #[cfg_attr(feature = "serde", serde(flatten))]
    variable: SpecimenVariableFields,
}
