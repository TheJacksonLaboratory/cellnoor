use macro_attributes::base_model;

#[base_model]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case", tag = "type"))]
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
