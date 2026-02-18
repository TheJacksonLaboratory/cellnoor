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
    pub directory: String,
    pub filename: String,
}

#[derive(Debug)]
pub struct ParsedMultipartFormField {
    content_type: String,
    directory: String,
    filename: String,
    content: axum::body::Bytes,
}

impl ParsedMultipartFormField {
    pub fn content_type(&self) -> &str {
        &self.content_type
    }

    pub fn directory(&self) -> &str {
        &self.directory
    }

    pub fn filename(&self) -> &str {
        &self.filename
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

        let (directory, filename) = extract_path(self.file_name())?;

        Ok(ParsedMultipartFormField {
            content_type,
            directory: directory.to_owned(),
            filename: filename.to_owned(),
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

fn extract_path(filename: Option<&str>) -> Result<(&str, &str), db::DataError> {
    let Some(path) = filename.map(Utf8Path::new) else {
        return Err(db::DataError::new_other("uploaded file must have filename"));
    };

    let (directory, filename) = {
        let mut ancestors = path.ancestors();
        ancestors.next().unwrap();
        let (Some(directory), Some(filename), Some("")) = (
            ancestors.next().map(Utf8Path::as_str),
            path.file_name(),
            ancestors.next().map(Utf8Path::as_str),
        ) else {
            return Err(db::DataError::new_other(
                "filename must be of the form 'directory/filename'",
            ));
        };

        (directory, filename)
    };

    Ok((directory, filename))
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
    fn filename_with_no_parent() {
        assert!(extract_path(Some("file")).is_err());
    }

    #[rstest]
    fn filename_with_too_many_parents() {
        assert!(extract_path(Some("grandparent/parent/file")).is_err());
    }

    #[rstest]
    fn root_filename() {
        assert!(extract_path(Some("/file")).is_err());
    }

    #[rstest]
    fn correct_filename() {
        let (directory, filename) = extract_path(Some("parent/file")).unwrap();

        assert_eq!((directory, filename), ("parent", "file"));
    }
}
