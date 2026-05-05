use macro_attributes::base_model;

#[base_model]
#[cfg_attr(feature = "schemars", schemars(inline))]
pub struct SimpleLinks {
    #[cfg_attr(feature = "serde", serde(rename = "self"))]
    pub self_: String,
}
