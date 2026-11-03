use askama::Template;
use askama_web::WebTemplate;
use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
    response::Redirect,
};
use axum_extra::extract::Form;
use cellnoor_types::{
    api_key::{ApiKey, ApiKeyQuery, ApiKeyUpdate, NewApiKey, SavedApiKeyRecord},
    bot_account::BotAccountQuery,
    nonempty::NonemptyString,
    person::PersonQuery,
};
use serde::Deserialize;

use crate::{
    auth::AuthUser,
    extract::JsonExtractor,
    handlers::{
        IdParam,
        api_keys::{create_api_key, index_api_keys, update_api_key},
        bot_accounts::index_bot_accounts,
        delete_resource,
        people::index_people,
    },
    state::AppState,
    ui::{
        BASE, Layout, UiError, User,
        format::person_options,
        params::{Labels, Opt, end_of_day, parse},
        redirect_back,
    },
};

#[derive(Template, WebTemplate)]
#[template(path = "api_keys.html")]
pub(super) struct ApiKeysPage {
    layout: Layout,
    api_keys: Vec<SavedApiKeyRecord>,
    bot_accounts: Vec<Opt>,
    owners: Vec<Opt>,
    secret: Option<String>,
}

impl ApiKeysPage {
    fn title(&self, api_key: &SavedApiKeyRecord) -> String {
        api_key
            .description
            .as_ref()
            .map_or_else(|| api_key.id.to_string(), ToString::to_string)
    }

    fn action(&self, api_key: &SavedApiKeyRecord, action: &str) -> String {
        format!("{BASE}/settings/api-keys/{}/{action}", api_key.id)
    }
}

pub(super) async fn index(
    State(state): State<AppState>,
    User(user): User,
) -> Result<ApiKeysPage, UiError> {
    page(&state, user, None).await
}

async fn page(
    state: &AppState,
    user: AuthUser,
    secret: Option<String>,
) -> Result<ApiKeysPage, UiError> {
    let (Json(api_keys), Json(bot_accounts), Json(people), layout) = tokio::try_join!(
        index_api_keys(
            State(state.clone()),
            user,
            JsonExtractor(ApiKeyQuery::default())
        ),
        index_bot_accounts(
            State(state.clone()),
            user,
            JsonExtractor(BotAccountQuery::default())
        ),
        index_people(
            State(state.clone()),
            user,
            JsonExtractor(PersonQuery::default())
        ),
        Layout::load(state, user),
    )?;

    let bot_accounts: Vec<_> = bot_accounts
        .iter()
        .map(|bot_account| {
            let id = bot_account.id.to_string();
            let name = bot_account
                .description
                .as_ref()
                .map_or_else(|| id.clone(), ToString::to_string);

            Opt::new(id, name)
        })
        .collect();

    Ok(ApiKeysPage {
        layout,
        api_keys,
        owners: bot_accounts
            .iter()
            .cloned()
            .chain(person_options(&people))
            .collect(),
        bot_accounts,
        secret,
    })
}

fn optional<T>(
    value: &str,
    parse: impl FnOnce(&str) -> Result<T, UiError>,
) -> Result<Option<T>, UiError> {
    if value.is_empty() {
        return Ok(None);
    }

    parse(value).map(Some)
}

#[derive(Deserialize)]
pub(super) struct CreateForm {
    description: String,
    owner_id: String,
    expires_on: String,
}

pub(super) async fn create(
    State(state): State<AppState>,
    User(user): User,
    Form(form): Form<CreateForm>,
) -> Result<ApiKeysPage, UiError> {
    let api_key = NewApiKey {
        description: NonemptyString::new(form.description).ok(),
        owner_id: optional(&form.owner_id, parse)?,
        expires_at: optional(&form.expires_on, end_of_day)?,
    };

    let Json(ApiKey { secret, .. }) =
        create_api_key(State(state.clone()), user, Some(Json(api_key))).await?;

    page(&state, user, Some(secret)).await
}

#[derive(Deserialize)]
pub(super) struct DescriptionForm {
    description: String,
    expires_at: String,
}

pub(super) async fn set_description(
    State(state): State<AppState>,
    User(user): User,
    Path(IdParam { id }): Path<IdParam>,
    headers: HeaderMap,
    Form(form): Form<DescriptionForm>,
) -> Result<Redirect, UiError> {
    let update = ApiKeyUpdate {
        description: NonemptyString::new(form.description).ok(),
        expires_at: optional(&form.expires_at, parse)?,
    };

    let Json(_) = update_api_key(State(state), user, Path(IdParam { id }), JsonExtractor(update)).await?;

    Ok(redirect_back(&headers))
}

#[derive(Deserialize)]
pub(super) struct ExpiresForm {
    expires_on: String,
    description: String,
}

pub(super) async fn set_expires_at(
    State(state): State<AppState>,
    User(user): User,
    Path(IdParam { id }): Path<IdParam>,
    headers: HeaderMap,
    Form(form): Form<ExpiresForm>,
) -> Result<Redirect, UiError> {
    let update = ApiKeyUpdate {
        description: NonemptyString::new(form.description).ok(),
        expires_at: Some(end_of_day(&form.expires_on)?),
    };

    let Json(_) = update_api_key(State(state), user, Path(IdParam { id }), JsonExtractor(update)).await?;

    Ok(redirect_back(&headers))
}

pub(super) async fn delete(
    State(state): State<AppState>,
    User(user): User,
    Path(IdParam { id }): Path<IdParam>,
    headers: HeaderMap,
) -> Result<Redirect, UiError> {
    let Json(_) = delete_resource::<ApiKeyUpdate>(State(state), user, Path(IdParam { id })).await?;

    Ok(redirect_back(&headers))
}
