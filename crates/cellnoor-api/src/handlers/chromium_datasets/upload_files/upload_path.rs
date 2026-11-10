use camino::{Utf8Component, Utf8Path};
use cellnoor_types::nonempty::NonemptyString;

use crate::handlers::chromium_datasets::upload_files::UploadFilesError;

#[derive(Debug, postgres_types::ToSql)]
#[postgres(transparent)]
pub(super) struct UploadPath(NonemptyString);

impl UploadPath {
    pub(super) fn new(multipart_filename: Option<&str>) -> Result<Self, UploadFilesError> {
        let path = multipart_filename
            .map(Utf8Path::new)
            .ok_or_else(filename_is_not_allowed_error)?;

        validate_filename_is_allowed(path)?;
        validate_every_component_is_normal(path)?;
        validate_file_is_at_top_level_or_in_sample_directory(path)?;

        Ok(NonemptyString::new(path.to_string())
            .map(Self)
            .expect("we validated the filename so it's not empty"))
    }

    pub(super) fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

static ALLOWED_FILENAMES: phf::Set<&str> = phf::phf_set! {
    "metrics_summary.csv",
    "qc_library_metrics.csv",
    "qc_report.html",
    "qc_sample_metrics.csv",
    "summary.csv",
    "summary.json",
    "web_summary.html",
};

fn validate_filename_is_allowed(path: &Utf8Path) -> Result<(), UploadFilesError> {
    let filename_is_allowed = path
        .file_name()
        .is_some_and(|filename| ALLOWED_FILENAMES.contains(filename));

    if !filename_is_allowed {
        return Err(filename_is_not_allowed_error());
    }

    Ok(())
}

fn filename_is_not_allowed_error() -> UploadFilesError {
    UploadFilesError::Filename {
        message: format!(
            "uploaded files must have a filename which is one of {:?}",
            ALLOWED_FILENAMES
        ),
    }
}

fn validate_every_component_is_normal(path: &Utf8Path) -> Result<(), UploadFilesError> {
    let every_component_is_normal = path
        .components()
        .all(|component| matches!(component, Utf8Component::Normal(_)));

    if !every_component_is_normal {
        return Err(UploadFilesError::Filename {
            message: "path must be relative and cannot contain '.' or '..'".to_owned(),
        });
    }

    Ok(())
}

fn validate_file_is_at_top_level_or_in_sample_directory(
    path: &Utf8Path,
) -> Result<(), UploadFilesError> {
    let components: Vec<_> = path.components().collect();

    match components.as_slice() {
        [_] | [Utf8Component::Normal("per_sample_outs"), _, _] => Ok(()),
        _ => Err(UploadFilesError::Filename {
            message: "files nested into a directory must be nested into a \
                      'per_sample_outs/sample_name/' directory"
                .to_owned(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::UploadPath;

    #[test]
    fn empty_filename() {
        UploadPath::new(Some("")).unwrap_err();
    }

    #[test]
    fn root_filename() {
        UploadPath::new(Some("/file")).unwrap_err();
    }

    #[test]
    fn correct_filenames() {
        let path = UploadPath::new(Some("metrics_summary.csv")).unwrap();

        assert_eq!(path.as_str(), "metrics_summary.csv");

        let path =
            UploadPath::new(Some("per_sample_outs/sample_name/metrics_summary.csv")).unwrap();

        assert_eq!(
            path.as_str(),
            "per_sample_outs/sample_name/metrics_summary.csv"
        );
    }

    #[test]
    fn non_normal_component() {
        UploadPath::new(Some("a/../web_summary.html")).unwrap_err();
    }
}
