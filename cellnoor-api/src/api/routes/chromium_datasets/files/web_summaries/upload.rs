use axum::extract::{Multipart, Path, State};
use cellnoor_models::IdParameter;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    api::routes::chromium_datasets::files::common::{FieldExt, ParsedMultipartFormField},
    db::{self, DbConnection},
    state::AppState,
};

static ALLOWED_CONTENT_TYPES: &[&str] = &["text/html"];

pub async fn upload_web_summary(
    _: State<AppState>,
    mut db_conn: DbConnection,
    Path(IdParameter { id }): Path<IdParameter>,
    mut request: Multipart,
) -> Result<(), db::Error> {
    let mut extracted_web_summaries = Vec::with_capacity(16);
    while let Some(field) = request
        .next_field()
        .await
        .map_err(|e| db::DataError::new_other(&e.to_string()))?
    {
        extracted_web_summaries.push(field.parse(ALLOWED_CONTENT_TYPES).await?);
    }

    insert_chromium_dataset_web_summaries(id, &extracted_web_summaries, &mut db_conn).await?;

    Ok(())
}

async fn insert_chromium_dataset_web_summaries(
    chromium_dataset_id: Uuid,
    web_summaries: &[ParsedMultipartFormField],
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<(), db::Error> {
    use cellnoor_schema::chromium_dataset_web_summaries::dsl::*;

    let insertables: Vec<_> = web_summaries
        .iter()
        .map(|d| {
            (
                dataset_id.eq(chromium_dataset_id),
                directory.eq(d.directory()),
                filename.eq(d.filename()),
                content.eq(d.content()),
            )
        })
        .collect();

    diesel::insert_into(chromium_dataset_web_summaries)
        .values(insertables)
        .execute(&mut db_conn)
        .await?;

    Ok(())
}
