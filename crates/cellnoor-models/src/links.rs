use macro_attributes::json;

#[json]
#[serde(untagged)]
enum Link {
    One(String),
    Many(Vec<String>),
}
