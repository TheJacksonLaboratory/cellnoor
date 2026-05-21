use macro_attributes::base_model;

use crate::id::Id;

#[base_model]
#[cfg_attr(feature = "schemars", schemars(inline))]
pub struct SimpleLinks {
    #[cfg_attr(feature = "serde", serde(rename = "self"))]
    pub self_: String,
}

impl SimpleLinks {
    pub fn from_str_and_id(s: &str, id: Id) -> Self {
        Self {
            self_: format!("{s}/{id}"),
        }
    }
}
