#![allow(clippy::result_large_err)]
use axum::extract::multipart::Field;
use camino::Utf8Path;
use schemars::JsonSchema;
use serde::Deserialize;
use uuid::Uuid;

use crate::db;

#[derive(Deserialize, JsonSchema)]
#[schemars(inline)]
pub struct FilePath {
    pub id: Uuid,
    pub path: Vec<String>,
}

#[derive(Debug)]
pub struct ParsedMultipartFormField {
    content_type: String,
    path: String,
    content: axum::body::Bytes,
}

impl ParsedMultipartFormField {
    pub fn content_type(&self) -> &str {
        &self.content_type
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn content(&self) -> &[u8] {
        &self.content
    }
}

pub trait FieldExt<'a> {
    async fn parse(
        self,
        allowed_content_types: &[&str],
    ) -> Result<ParsedMultipartFormField, db::DataError>;
}

impl<'a> FieldExt<'a> for Field<'a> {
    async fn parse(
        self,
        allowed_content_types: &[&str],
    ) -> Result<ParsedMultipartFormField, db::DataError> {
        let content_type = extract_content_type(self.content_type(), allowed_content_types)?;

        let path = extract_path(self.file_name())?;

        Ok(ParsedMultipartFormField {
            content_type,
            path: path.to_owned(),
            content: self
                .bytes()
                .await
                .map_err(|e| db::DataError::new_other(&e.body_text()))?,
        })
    }
}

fn extract_content_type(
    content_type: Option<&str>,
    allowed_content_types: &[&str],
) -> Result<String, db::DataError> {
    let Some(content_type) = content_type else {
        return Err(db::DataError::new_other(
            "file-upload must have content-type",
        ))?;
    };

    if !allowed_content_types.contains(&content_type) {
        return Err(db::DataError::new_other(&format!(
            "file-upload must have one of the following content-types: {allowed_content_types:?}"
        )));
    }

    Ok(content_type.to_owned())
}

fn extract_path(filename: Option<&str>) -> Result<&str, db::DataError> {
    if filename.is_some_and(str::is_empty) {
        return Err(db::DataError::new_other("filename cannot be empty"));
    }

    let Some(path) = filename.map(Utf8Path::new) else {
        return Err(db::DataError::new_other("uploaded file must have filename"));
    };

    if path.is_absolute() {
        return Err(db::DataError::new_other(
            "file cannot be in the root directory",
        ));
    }

    Ok(path.as_str())
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::extract_path;

    #[rstest]
    fn empty_filename() {
        assert!(extract_path(Some("")).is_err());
    }

    #[rstest]
    fn root_filename() {
        assert!(extract_path(Some("/file")).is_err());
    }

    #[rstest]
    fn correct_filename() {
        let path = extract_path(Some("parent/file")).unwrap();

        assert_eq!(path, "parent/file");

        let path = extract_path(Some("grandparent/parent/file")).unwrap();

        assert_eq!(path, "grandparent/parent/file");
    }
}
