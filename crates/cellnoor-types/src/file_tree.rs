use macro_attributes::base_model;

#[base_model]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case", tag = "type"))]
#[cfg_attr(feature = "schemars", schemars(rename = "{T}Tree"))]
pub enum FileTreeNode<T> {
    File {
        name: String,
        raw_file_link: String,
        parsed_contents: Option<T>,
    },
    Directory {
        name: String,
        children: Vec<FileTreeNode<T>>,
    },
}
