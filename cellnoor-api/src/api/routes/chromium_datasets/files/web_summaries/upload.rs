use axum::{
    Json,
    extract::{Multipart, State},
    http::StatusCode,
};
use cellnoor_models::chromium_dataset::ChromiumDatasetIdWebSummaries;
use diesel::prelude::*;

use crate::{
    api::{
        extract::auth::AuthenticatedUser,
        routes::{
            ApiResponse,
            chromium_datasets::files::common::{FieldExt, ParsedMultipartFormField},
            inner_handler,
        },
    },
    db,
    state::AppState,
};

static ALLOWED_CONTENT_TYPES: &[&str] = &["text/html"];

pub async fn upload_web_summary(
    chromium_dataset_id: ChromiumDatasetIdWebSummaries,
    state: State<AppState>,
    user: AuthenticatedUser,
    mut request: Multipart,
) -> ApiResponse<()> {
    let mut extracted_web_summaries = Vec::with_capacity(16);
    while let Some(field) = request.next_field().await? {
        extracted_web_summaries.push(field.parse(ALLOWED_CONTENT_TYPES).await?);
    }

    let _ = inner_handler(state, user, (chromium_dataset_id, extracted_web_summaries)).await?;
    Ok((StatusCode::CREATED, Json(())))
}

impl db::Operation<()> for (ChromiumDatasetIdWebSummaries, Vec<ParsedMultipartFormField>) {
    fn execute(self, db_conn: &mut diesel::PgConnection) -> Result<(), db::Error> {
        use cellnoor_schema::chromium_dataset_web_summaries::dsl::*;

        let (ds_id, data) = self;
        let insertables: Vec<_> = data
            .iter()
            .map(|d| {
                (
                    dataset_id.eq(ds_id),
                    directory.eq(d.directory()),
                    filename.eq(d.filename()),
                    content.eq(d.content()),
                )
            })
            .collect();

        diesel::insert_into(chromium_dataset_web_summaries)
            .values(insertables)
            .execute(db_conn)?;

        Ok(())
    }
}
