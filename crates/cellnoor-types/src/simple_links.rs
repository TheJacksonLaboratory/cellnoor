use macro_attributes::base_model;
use uuid::Uuid;

#[base_model]
#[cfg_attr(feature = "schemars", schemars(inline))]
pub struct SimpleLinks {
    #[cfg_attr(feature = "serde", serde(rename = "self"))]
    pub self_: String,
}

impl SimpleLinks {
    pub fn from_str_and_id(s: &str, id: Uuid) -> Self {
        Self {
            self_: format!("{s}/{id}"),
        }
    }
}
