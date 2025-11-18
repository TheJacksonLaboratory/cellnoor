use axum::{Json, Router, extract::State, http::StatusCode};
use axum_extra::routing::{RouterExt, TypedPath};
use scamplers_models::{
    institution::{self, Institution, InstitutionId},
    lab::{self, Lab},
    person::{self, Person, PersonId, PersonSummary},
};
use serde_qs::axum::QsQuery;

use crate::{
    api::{
        error::ErrorResponse,
        extract::{ValidJson, auth::AuthenticatedUser},
    },
    db,
    state::AppState,
};

mod chromium_datasets;
mod chromium_runs;
mod institutions;
mod labs;
mod nucleic_acids;
mod people;
mod sequencing_runs;
mod specimens;
mod suspensions;

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .typed_post(create_institution)
        .typed_get(fetch_institution)
        .typed_get(list_institutions)
        .typed_post(create_person)
        .typed_get(fetch_person)
        .typed_get(list_people)
        .typed_post(create_lab)
    // .typed_get(fetch_labs)
    // .typed_get(list_labs)
}

type ApiResponse<T> = Result<(StatusCode, Json<T>), super::error::ErrorResponse>;

#[derive(TypedPath)]
#[typed_path("/institutions")]
struct InstitutionsEndpoint;

async fn create_institution(
    _: InstitutionsEndpoint,
    state: State<AppState>,
    user: AuthenticatedUser,
    ValidJson(request): ValidJson<institution::Creation>,
) -> ApiResponse<Institution> {
    let item = inner_handler(state, user, request).await?;
    Ok((StatusCode::CREATED, item))
}

async fn fetch_institution(
    request: InstitutionId,
    state: State<AppState>,
    user: AuthenticatedUser,
) -> ApiResponse<Institution> {
    let item = inner_handler(state, user, request).await?;
    Ok((StatusCode::FOUND, item))
}

async fn list_institutions(
    _: InstitutionsEndpoint,
    state: State<AppState>,
    user: AuthenticatedUser,
    serde_qs::axum::OptionalQsQuery(request): serde_qs::axum::OptionalQsQuery<institution::Query>,
) -> ApiResponse<Vec<Institution>> {
    let items = inner_handler(state, user, request.unwrap_or_default()).await?;
    Ok((StatusCode::OK, items))
}

#[derive(serde::Deserialize, TypedPath)]
#[typed_path("/institutions/{id}/members")]
struct InstitutionMembersEndpoint(InstitutionId);

#[derive(TypedPath)]
#[typed_path("/people")]
struct PeopleEndpoint;

async fn create_person(
    _: PeopleEndpoint,
    state: State<AppState>,
    user: AuthenticatedUser,
    ValidJson(request): ValidJson<person::Creation>,
) -> ApiResponse<Person> {
    let item = inner_handler(state, user, request).await?;
    Ok((StatusCode::CREATED, item))
}

async fn fetch_person(
    request: PersonId,
    state: State<AppState>,
    user: AuthenticatedUser,
) -> ApiResponse<Person> {
    let item = inner_handler(state, user, request).await?;
    Ok((StatusCode::OK, item))
}

async fn list_people(
    _: PeopleEndpoint,
    state: State<AppState>,
    user: AuthenticatedUser,
    QsQuery(request): QsQuery<person::Query>,
) -> ApiResponse<Vec<PersonSummary>> {
    let items = inner_handler(state, user, request).await?;
    Ok((StatusCode::OK, items))
}

#[derive(TypedPath)]
#[typed_path("/labs")]
struct LabsEndpoint;

async fn create_lab(
    _: LabsEndpoint,
    state: State<AppState>,
    user: AuthenticatedUser,
    ValidJson(request): ValidJson<lab::Creation>,
) -> ApiResponse<Lab> {
    let item = inner_handler(state, user, request).await?;
    Ok((StatusCode::CREATED, item))
}

async fn inner_handler<Request, Response>(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    request: Request,
) -> Result<Json<Response>, ErrorResponse>
where
    Request: std::fmt::Debug + db::Operation<Response> + Send + serde::Serialize + 'static,
    Response: Send + 'static,
{
    tracing::info!("{request:?}");

    let db_conn = state.db_conn().await?;

    let response = db_conn
        .interact(move |db_conn| request.execute_as_user(user.id(), db_conn))
        .await??;

    Ok(Json(response))
}
