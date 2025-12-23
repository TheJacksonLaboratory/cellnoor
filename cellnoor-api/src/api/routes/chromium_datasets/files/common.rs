#![allow(clippy::result_large_err)]
use axum::{extract::multipart::Field, http::StatusCode};
use camino::Utf8Path;

use crate::api::{self, ErrorResponse};

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
    ) -> Result<ParsedMultipartFormField, ErrorResponse>;
}

impl<'a> FieldExt<'a> for Field<'a> {
    async fn parse(
        self,
        allowed_content_types: &[&str],
    ) -> Result<ParsedMultipartFormField, ErrorResponse> {
        let content_type = extract_content_type(self.content_type(), allowed_content_types)?;

        let (directory, filename) = extract_path(self.file_name())?;

        Ok(ParsedMultipartFormField {
            content_type,
            directory: directory.to_string(),
            filename: filename.to_string(),
            content: self.bytes().await?,
        })
    }
}

fn extract_content_type(
    content_type: Option<&str>,
    allowed_content_types: &[&str],
) -> Result<String, ErrorResponse> {
    let Some(content_type) = content_type else {
        return Err(ErrorResponse {
            status: StatusCode::NOT_ACCEPTABLE.as_u16(),
            public_error: api::Error::MalformedRequest {
                message: "file-upload must have content-type".to_owned(),
            },
            internal_error: None,
        });
    };

    if !allowed_content_types.contains(&content_type) {
        return Err(ErrorResponse {
            status: StatusCode::NOT_ACCEPTABLE.as_u16(),
            public_error: api::Error::MalformedRequest {
                message: format!(
                    "file-upload must have one of the following content-types: \
                     {allowed_content_types:?}"
                ),
            },
            internal_error: None,
        });
    }

    Ok(content_type.to_owned())
}

fn extract_path(filename: Option<&str>) -> Result<(&str, &str), ErrorResponse> {
    let Some(path) = filename.map(Utf8Path::new) else {
        return Err(ErrorResponse {
            status: StatusCode::UNPROCESSABLE_ENTITY.as_u16(),
            public_error: api::Error::MalformedRequest {
                message: "file-upload must have filename".to_owned(),
            },
            internal_error: None,
        });
    };

    let (directory, filename) = {
        let mut ancestors = path.ancestors();
        ancestors.next().unwrap();
        let (Some(directory), Some(filename), Some("")) = (
            ancestors.next().map(Utf8Path::as_str),
            path.file_name(),
            ancestors.next().map(Utf8Path::as_str),
        ) else {
            return Err(ErrorResponse {
                status: StatusCode::UNPROCESSABLE_ENTITY.as_u16(),
                public_error: api::Error::MalformedRequest {
                    message: "filename must be of the form 'directory/filename'".to_owned(),
                },
                internal_error: None,
            });
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
