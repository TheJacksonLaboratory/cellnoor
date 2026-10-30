use std::str::FromStr;

use axum::extract::multipart::Field;
use strum::VariantNames;

use crate::handlers::chromium_datasets::upload_files::UploadFilesError;

#[derive(
    Debug, Clone, Copy, strum::EnumString, strum::AsRefStr, PartialEq, strum::VariantNames,
)]
pub(super) enum AllowedContentType {
    #[strum(serialize = "text/csv", serialize = "text/csv;charset=utf-8")]
    Csv,
    #[strum(serialize = "text/html", serialize = "text/html;charset=utf-8")]
    Html,
    #[strum(
        serialize = "application/json",
        serialize = "application/json;charset=utf-8"
    )]
    Json,
}

impl AllowedContentType {
    pub(super) fn from_multipart_form_field(field: &Field<'_>) -> Result<Self, UploadFilesError> {
        field
            .content_type()
            .map(AllowedContentType::from_str)
            .ok_or(UploadFilesError::ContentType {
                message: "file-upload must have content type".to_owned(),
            })?
            .map_err(|_| UploadFilesError::ContentType {
                message: format!("content-type must be one of: {:?}", Self::VARIANTS),
            })
    }
}
