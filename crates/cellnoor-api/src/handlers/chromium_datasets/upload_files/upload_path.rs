use camino::{Utf8Component, Utf8Path};
use cellnoor_types::nonempty::NonemptyString;

use crate::handlers::chromium_datasets::upload_files::UploadFilesError;

#[derive(Debug, postgres_types::ToSql)]
#[postgres(transparent)]
pub(super) struct UploadPath(NonemptyString);

impl UploadPath {
    pub(super) fn new(filename: Option<&str>) -> Result<Self, UploadFilesError> {
        const ALLOWED_FILENAMES: [&str; 7] = [
            "metrics_summary.csv",
            "qc_library_metrics.csv",
            "qc_report.html",
            "qc_sample_metrics.csv",
            "summary.csv",
            "summary.json",
            "web_summary.html",
        ];

        let filename_error = Err(UploadFilesError::Filename {
            message: format!(
                "uploaded files must have a filename which is one of {:?}",
                ALLOWED_FILENAMES
            ),
        });

        let Some(path) = filename.map(Utf8Path::new) else {
            return filename_error;
        };

        if path
            .file_name()
            .is_none_or(|f| !ALLOWED_FILENAMES.contains(&f))
        {
            return filename_error;
        }

        let components: Vec<_> = path.components().collect();

        if !components
            .iter()
            .all(|c| matches!(c, Utf8Component::Normal(_)))
        {
            return Err(UploadFilesError::Filename {
                message: "path must be relative and cannot contain '.' or '..'".to_owned(),
            });
        }

        match components.as_slice() {
            [_] | [Utf8Component::Normal("per_sample_outs"), _, _] => {
                Ok(NonemptyString::new(path.to_string()).map(Self).unwrap())
            }
            _ => Err(UploadFilesError::Filename {
                message: "files nested into a directory must be nested into a \
                          'per_sample_outs/sample_name/' directory"
                    .to_owned(),
            }),
        }
    }

    pub(super) fn as_str(&self) -> &str {
        self.0.as_ref()
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
