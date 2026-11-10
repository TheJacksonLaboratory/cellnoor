#[cfg(any(test, feature = "dev"))]
use axum::{
    body::Body,
    extract::{DefaultBodyLimit, FromRequest},
    http::{HeaderMap, HeaderValue, Request},
};
use axum::extract::{Multipart, Path, State};
use camino::Utf8Path;
#[cfg(any(test, feature = "dev"))]
use reqwest::{
    header::CONTENT_TYPE,
    multipart::{Form, Part},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::Pool,
    handlers::{
        IdParam,
        chromium_datasets::{
            dataset_directory::FinalDirectory,
            upload_files::{stage_files::StagedUpload, write_to_db::write_files_to_db},
        },
    },
    state::AppState,
};

pub use error::UploadFilesError;

mod allowed_content_type;
mod error;
mod stage_files;
mod upload_path;
pub(super) mod write_to_db;

pub async fn upload_files(
    State(app_state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id: dataset_id }): Path<IdParam>,
    files: Multipart,
) -> Result<(), UploadFilesError> {
    write_files_to_disk_and_db(
        &app_state.db_pool,
        &app_state.static_files_dir,
        user,
        dataset_id,
        files,
    )
    .await
}

// The actual implementation lives in this separate function so we can call this
// in tests without needing a full AppState
async fn write_files_to_disk_and_db(
    db_pool: &Pool,
    static_files_dir: &Utf8Path,
    user: AuthUser,
    dataset_id: Uuid,
    files: Multipart,
) -> Result<(), UploadFilesError> {
    let dataset_directory = FinalDirectory::new(static_files_dir, dataset_id);

    // No database client is held here, because parsing and compressing files is
    // CPU-heavy
    let staged_upload = StagedUpload::from_multipart(dataset_directory, files).await?;

    db_pool
        .in_transaction(user, async |tx| {
            let overwritten_file_paths =
                write_files_to_db(tx, dataset_id, staged_upload.files()).await?;

            staged_upload.move_files_into_final_directory(&overwritten_file_paths)?;

            Ok(())
        })
        .await
}

#[cfg(any(test, feature = "dev"))]
pub async fn write_test_dataset_files(
    db_pool: &Pool,
    dataset_id: Uuid,
    static_files_dir: &Utf8Path,
) {
    const MULTI_METRICS: &[u8] = include_bytes!(
        "upload_files/test-data/\
         4plex_rat_liver_kidney_lgintest_testis_CF_RatLiver_A-A03_metrics_summary.csv"
    );

    let csv_header: HeaderMap = [(CONTENT_TYPE, HeaderValue::from_static("text/csv"))]
        .into_iter()
        .collect();

    let body = Form::new()
        .part(
            "web_summary",
            Part::bytes(include_bytes!(
                "upload_files/test-data/\
                 4plex_rat_liver_kidney_lgintest_testis_CF_RatLiver_A-A03_web_summary.html"
            ))
            .headers(
                [(CONTENT_TYPE, HeaderValue::from_static("text/html"))]
                    .into_iter()
                    .collect(),
            )
            .file_name("per_sample_outs/sample1/web_summary.html"),
        )
        .part(
            "qc_sample_metrics",
            Part::bytes(include_bytes!(
                "upload_files/test-data/\
                 SOD1_G93A_mouse_spinal_cord_P112_specimen_1_Multiplex_qc_sample_metrics.csv"
            ))
            .headers(csv_header.clone())
            .file_name("per_sample_outs/sample1/qc_sample_metrics.csv"),
        )
        .part(
            "summary1",
            Part::bytes(b"string_field,number_field\nstring_value,0")
                .headers(csv_header.clone())
                .file_name("qc_library_metrics.csv"),
        )
        .part(
            "summary2",
            Part::bytes(b"string_field,other_number_field\nstring_value,10")
                .headers(csv_header.clone())
                .file_name("summary.csv"),
        )
        .part(
            "metrics1",
            Part::bytes(MULTI_METRICS)
                .headers(csv_header.clone())
                .file_name("per_sample_outs/sample1/metrics_summary.csv"),
        )
        .part(
            "metrics2",
            Part::bytes(MULTI_METRICS)
                .headers(csv_header)
                .file_name("per_sample_outs/sample2/metrics_summary.csv"),
        )
        .part(
            "json_metrics",
            Part::bytes(
                br#"{"string_key": "string_value", "number": 0, "nested": {"is_this_cool": true}}"#,
            )
            .headers(
                [(CONTENT_TYPE, HeaderValue::from_static("application/json"))]
                    .into_iter()
                    .collect(),
            )
            .file_name("per_sample_outs/sample1/summary.json"),
        );

    // Thanks claude
    let request = reqwest::Client::new()
        .post("http://localhost")
        .multipart(body)
        .build()
        .unwrap();
    let mut request = Request::<reqwest::Body>::try_from(request)
        .unwrap()
        .map(Body::new);

    DefaultBodyLimit::disable().apply(&mut request);

    write_files_to_disk_and_db(
        db_pool,
        static_files_dir,
        AuthUser::dev_admin(),
        dataset_id,
        Multipart::from_request(request, &()).await.unwrap(),
    )
    .await
    .unwrap();
}

#[cfg(test)]
mod tests {
    use camino::Utf8Path;
    use pretty_assertions::assert_eq;
    use tempfile::tempdir;

    use crate::{
        handlers::{
            chromium_datasets::{insert_test_chromium_dataset, write_test_dataset_files},
            chromium_runs::TestChromiumRunKind,
        },
        state::dev_util::{db_client_as_admin, test_db_pool},
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn upload_dataset_files() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, ds) = insert_test_chromium_dataset(&tx, TestChromiumRunKind::Standard, |_| ())
            .await
            .unwrap();
        tx.commit().await.unwrap();

        let db_pool = test_db_pool();
        let static_files_dir = tempdir().unwrap();
        let static_files_dir = Utf8Path::from_path(static_files_dir.path()).unwrap();

        write_test_dataset_files(&db_pool, *ds.record.id, static_files_dir).await;
        write_test_dataset_files(&db_pool, *ds.record.id, static_files_dir).await;

        let all_datasets_directory = static_files_dir.join("chromium-datasets");
        let dataset_directory = all_datasets_directory.join(ds.record.id.to_string());

        assert_eq!(
            sorted_entry_names(&all_datasets_directory),
            [ds.record.id.to_string()]
        );
        assert_eq!(
            sorted_entry_names(&dataset_directory),
            [
                "per_sample_outs",
                "qc_library_metrics.csv",
                "qc_library_metrics.csv.zst",
                "summary.csv",
                "summary.csv.zst",
            ]
        );
        assert_eq!(
            sorted_entry_names(&dataset_directory.join("per_sample_outs/sample1")),
            [
                "metrics_summary.csv",
                "metrics_summary.csv.zst",
                "qc_sample_metrics.csv",
                "qc_sample_metrics.csv.zst",
                "summary.json",
                "summary.json.zst",
                "web_summary.html",
                "web_summary.html.zst",
            ]
        );
    }

    pub(super) fn sorted_entry_names(directory_path: &Utf8Path) -> Vec<String> {
        let mut entry_names: Vec<_> = directory_path
            .read_dir_utf8()
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_owned())
            .collect();
        entry_names.sort();

        entry_names
    }
}
