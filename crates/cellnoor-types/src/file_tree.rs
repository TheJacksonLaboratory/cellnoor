use macro_attributes::base_model;

#[base_model]
#[cfg_attr(feature = "schemars", schemars(rename = "{T}FileTree"))]
pub struct FileTreeNode<T> {
    pub path: String,
    pub name: String,
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub kind: FileTreeNodeKind<T>,
}

#[base_model]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case", tag = "type"))]
#[cfg_attr(feature = "schemars", schemars(rename = "{T}FileTreeNodeKind"))]
pub enum FileTreeNodeKind<T> {
    File {
        raw_file_link: String,
        parsed_contents: Option<T>,
    },
    Directory {
        children: Vec<FileTreeNode<T>>,
    },
}
