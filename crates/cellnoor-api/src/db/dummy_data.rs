use axum::{Json, extract::State};
use cellnoor_types::{
    person::{Account, NewPerson, PersonSimpleFields},
    project::NewProject,
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    extract::JsonExtractor,
    handlers::people::create_person,
    state::{AppState, dev_util::ToNonemptyString},
};

pub async fn populate_dummy_data(app_state: AppState) {
    let people = (0..100).map(|i| NewPerson {
        simple: PersonSimpleFields {
            name: format!("person{i}").to_nonempty_string(),
            institution_id: Uuid::nil(),
            is_staff: false,
            orcid: None,
        },
        permissions_to_grant: vec![],
        account: Account::None {
            email: format!("person{i}@jax.org").to_nonempty_string(),
        },
    });

    let people = create_entities(create_person, app_state, people).await;
}

async fn create_entities<Req, Resp, E: std::error::Error + Sync + Send + 'static>(
    create: impl AsyncFn(State<AppState>, AuthUser, JsonExtractor<Req>) -> Result<Json<Resp>, E>,
    app_state: AppState,
    data: impl IntoIterator<Item = Req>,
) -> Vec<Resp> {
    let mut responses = Vec::with_capacity(500);

    for d in data {
        let response = create(
            State(app_state.clone()),
            AuthUser::admin(),
            JsonExtractor(d),
        )
        .await
        .unwrap();
        responses.push(response.0);
    }

    responses
}
