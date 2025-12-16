use axum::{
    Json,
    extract::{Multipart, State, multipart::Field},
    http::StatusCode,
};
use diesel::prelude::*;
use scamplers_models::chromium_dataset::ChromiumDatasetIdWebSummaries;
use scamplers_schema::chromium_dataset_web_summaries;

use crate::{
    api::{
        self, ErrorResponse,
        extract::auth::AuthenticatedUser,
        routes::{ApiResponse, inner_handler},
    },
    db,
    state::AppState,
};

pub async fn upload_web_summary(
    chromium_dataset_id: ChromiumDatasetIdWebSummaries,
    state: State<AppState>,
    user: AuthenticatedUser,
    mut request: Multipart,
) -> ApiResponse<()> {
    let mut extracted_web_summaries = Vec::with_capacity(16);
    while let Some(field) = request.next_field().await.unwrap() {
        extracted_web_summaries.push(WebSummary::from_field(chromium_dataset_id, field).await?);
    }

    let _ = inner_handler(state, user, extracted_web_summaries).await?;
    Ok((StatusCode::CREATED, Json(())))
}

#[derive(Debug, Insertable)]
#[diesel(table_name = chromium_dataset_web_summaries, check_for_backend(Pg))]
struct WebSummary {
    dataset_id: ChromiumDatasetIdWebSummaries,
    filename: String,
    content: Vec<u8>,
}

impl WebSummary {
    async fn from_field(
        dataset_id: ChromiumDatasetIdWebSummaries,
        field: Field<'_>,
    ) -> Result<Self, ErrorResponse> {
        if field.content_type() != Some("text/html") {
            return Err(ErrorResponse {
                status: StatusCode::UNPROCESSABLE_ENTITY.as_u16(),
                public_error: api::Error::MalformedRequest {
                    message: "expected HTML".to_owned(),
                },
                internal_error: None,
            });
        }

        Ok(Self {
            dataset_id,
            filename: field
                .file_name()
                .ok_or(ErrorResponse {
                    status: StatusCode::UNPROCESSABLE_ENTITY.as_u16(),
                    public_error: api::Error::MalformedRequest {
                        message: "Chromium web summary must have filename".to_owned(),
                    },
                    internal_error: None,
                })?
                .to_owned(),
            content: field.bytes().await?.into(),
        })
    }
}

impl db::Operation<()> for Vec<WebSummary> {
    fn execute(self, db_conn: &mut diesel::PgConnection) -> Result<(), db::Error> {
        diesel::insert_into(chromium_dataset_web_summaries::table)
            .values(self)
            .execute(db_conn)?;

        Ok(())
    }
}
