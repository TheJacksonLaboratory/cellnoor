use askama::Template;
use askama_web::WebTemplate;
use axum::{
    Json, Router,
    extract::{FromRequestParts, Path, Request, State},
    http::{HeaderMap, Method, StatusCode, Uri, header, request::Parts},
    middleware::{self, Next},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
};
use axum_extra::extract::Form;
use cellnoor_types::{
    operator::UuidOperator,
    permission::{Action, Permission, Resource},
    person::{PersonPredicate, PersonQuery},
};
use serde::Deserialize;
use strum::VariantArray;
use uuid::Uuid;

use crate::{
    auth::{AuthError, AuthUser},
    db::DbError,
    extract::JsonExtractor,
    handlers::{
        IdParam,
        people::{PersonError, index_people_with_permissions},
        permissions::{grant_permissions, revoke_permissions},
    },
    state::AppState,
    ui::format::Choice,
};

mod api_keys;
mod bot_accounts;
mod chromium_datasets;
mod format;
mod params;
mod people;
mod projects;

// Caddy strips `/api` before proxying to this app (see caddy/Caddyfile)
pub(crate) const BASE: &str = "/api/ui";

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(placeholder))
        .route("/specimens", get(placeholder))
        .route("/specimens/{id}", get(placeholder))
        .route("/libraries", get(placeholder))
        .route("/projects", get(projects::index))
        .route("/projects/{id}", get(placeholder))
        .route("/projects/{id}/members", post(projects::update_members))
        .route("/chromium-datasets", get(chromium_datasets::index))
        .route("/chromium-datasets/{id}", get(chromium_datasets::show))
        .route("/settings/people", get(people::index))
        .route("/settings/people/{id}/staff", post(people::set_staff))
        .route(
            "/settings/bot-accounts",
            get(bot_accounts::index).post(bot_accounts::create),
        )
        .route(
            "/settings/bot-accounts/{id}/description",
            post(bot_accounts::set_description),
        )
        .route(
            "/settings/bot-accounts/{id}/staff",
            post(bot_accounts::set_staff),
        )
        .route(
            "/settings/bot-accounts/{id}/members",
            post(bot_accounts::update_members),
        )
        .route(
            "/settings/bot-accounts/{id}/delete",
            post(bot_accounts::delete),
        )
        .route(
            "/settings/api-keys",
            get(api_keys::index).post(api_keys::create),
        )
        .route(
            "/settings/api-keys/{id}/description",
            post(api_keys::set_description),
        )
        .route(
            "/settings/api-keys/{id}/expires-at",
            post(api_keys::set_expires_at),
        )
        .route("/settings/api-keys/{id}/delete", post(api_keys::delete))
        .route("/principals/{id}/permissions", post(set_permissions))
        .route("/assets/{file}", get(asset))
        .layer(middleware::from_fn(reject_cross_origin_writes))
}

struct User(AuthUser);

impl FromRequestParts<AppState> for User {
    type Rejection = UiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        match AuthUser::from_request_parts(parts, state).await {
            Ok(user) => Ok(Self(user)),
            Err(AuthError::Db(error)) => Err(UiError::Db(error)),
            Err(_) => {
                let callback = match (&parts.method, parts.uri.path_and_query()) {
                    (&Method::GET, Some(path)) => format!("{BASE}{path}"),
                    _ => BASE.to_owned(),
                };

                Err(UiError::Unauthenticated { callback })
            }
        }
    }
}

struct Layout {
    can_update_people: bool,
}

impl Layout {
    async fn load(state: &AppState, user: AuthUser) -> Result<Self, DbError> {
        let me = PersonQuery::from_filter(PersonPredicate::Id(UuidOperator::Eq(user.id())));
        let Json(people) =
            index_people_with_permissions(State(state.clone()), user, JsonExtractor(me)).await?;

        let update_people = Permission {
            resource: Resource::Person,
            action: Action::Update,
        };

        Ok(Self {
            can_update_people: people.iter().any(|p| p.permissions.contains(&update_people)),
        })
    }
}

#[derive(Debug)]
enum UiError {
    Unauthenticated { callback: String },
    BadRequest(String),
    CrossOrigin,
    Db(DbError),
    Person(PersonError),
}

impl From<DbError> for UiError {
    fn from(error: DbError) -> Self {
        Self::Db(error)
    }
}

impl From<PersonError> for UiError {
    fn from(error: PersonError) -> Self {
        Self::Person(error)
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "sign_in.html")]
struct SignIn {
    callback: String,
}

#[derive(Template, WebTemplate)]
#[template(path = "error.html")]
struct ErrorPage {
    message: String,
    details: Option<String>,
}

impl IntoResponse for UiError {
    fn into_response(self) -> Response {
        let (status, message, details) = match self {
            Self::Unauthenticated { callback } => {
                return (StatusCode::UNAUTHORIZED, SignIn { callback }).into_response();
            }
            Self::BadRequest(message) => (StatusCode::BAD_REQUEST, message, None),
            Self::CrossOrigin => (
                StatusCode::FORBIDDEN,
                "Cross-origin requests are not allowed".to_owned(),
                None,
            ),
            Self::Db(error) => {
                tracing::error!("{error}");
                (error.status(), db_error_message(&error), details(&error))
            }
            Self::Person(PersonError::Db(error)) => return Self::Db(error).into_response(),
            Self::Person(PersonError::InvalidEmail { email }) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                format!("{email} is not a valid email address"),
                None,
            ),
        };

        (status, ErrorPage { message, details }).into_response()
    }
}

fn db_error_message(error: &DbError) -> String {
    match error {
        DbError::ResourceNotFound => "Item not found (or you don't have access to it)".to_owned(),
        DbError::InvalidReference {
            referencing_resource,
            referencing_field,
        } => format!(
            "Invalid {} for {}",
            format::format_enum(referencing_field),
            format::format_enum(referencing_resource)
        ),
        DbError::DataConstraint { message, .. } => message.clone(),
        DbError::PermissionDenied { .. } => "Permission denied".to_owned(),
        DbError::DuplicateResource { .. } | DbError::Other { .. } => {
            "Something went wrong".to_owned()
        }
    }
}

fn details(error: &impl serde::Serialize) -> Option<String> {
    serde_json::to_string_pretty(error).ok()
}

fn redirect_back(headers: &HeaderMap) -> Redirect {
    let referer = headers
        .get(header::REFERER)
        .and_then(|referer| referer.to_str().ok()?.parse::<Uri>().ok());

    let path = referer
        .as_ref()
        .and_then(Uri::path_and_query)
        .map(|path| path.as_str())
        .filter(|path| path.starts_with(BASE));

    Redirect::to(path.unwrap_or(BASE))
}

// Unlike JSON, HTML forms can be posted cross-site, so writes must prove they're same-origin
// (https://cheatsheetseries.owasp.org/cheatsheets/Cross-Site_Request_Forgery_Prevention_Cheat_Sheet.html)
async fn reject_cross_origin_writes(request: Request, next: Next) -> Response {
    let method = request.method();

    if method == Method::GET || method == Method::HEAD || is_same_origin(&request) {
        next.run(request).await
    } else {
        UiError::CrossOrigin.into_response()
    }
}

fn is_same_origin(request: &Request) -> bool {
    let headers = request.headers();

    if let Some(site) = headers.get("sec-fetch-site") {
        return site == "same-origin";
    }

    let host = headers
        .get(header::HOST)
        .and_then(|host| host.to_str().ok())
        .or_else(|| request.uri().authority().map(|a| a.as_str()));

    let origin = headers
        .get(header::ORIGIN)
        .and_then(|origin| origin.to_str().ok()?.parse::<Uri>().ok());

    match (host, origin.as_ref().and_then(Uri::authority)) {
        (Some(host), Some(origin)) => origin.as_str() == host,
        _ => false,
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "placeholder.html")]
struct Placeholder {
    layout: Layout,
}

async fn placeholder(State(state): State<AppState>, User(user): User) -> Result<Placeholder, UiError> {
    Ok(Placeholder {
        layout: Layout::load(&state, user).await?,
    })
}

struct PermissionGrid {
    principal_id: Uuid,
    granted: Vec<String>,
    actions: Vec<String>,
    rows: Vec<(String, Vec<PermissionCell>)>,
}

struct PermissionCell {
    value: String,
    label: String,
    checked: bool,
}

impl PermissionGrid {
    fn new(principal_id: Uuid, granted: &[Permission]) -> Self {
        let rows = Resource::VARIANTS
            .iter()
            .map(|&resource| {
                let cells = Action::VARIANTS
                    .iter()
                    .map(|&action| {
                        let permission = Permission { resource, action };

                        PermissionCell {
                            value: permission_value(&permission),
                            label: format!("{} {}", action.label(), resource.label()),
                            checked: granted.contains(&permission),
                        }
                    })
                    .collect();

                (resource.label(), cells)
            })
            .collect();

        Self {
            principal_id,
            granted: granted.iter().map(permission_value).collect(),
            actions: Action::VARIANTS.iter().map(Choice::label).collect(),
            rows,
        }
    }
}

fn permission_value(Permission { resource, action }: &Permission) -> String {
    format!("{}:{}", resource.as_ref(), action.as_ref())
}

fn parse_permission(value: &str) -> Result<Permission, UiError> {
    let invalid = || UiError::BadRequest(format!("invalid permission `{value}`"));

    let (resource, action) = value.split_once(':').ok_or_else(invalid)?;

    Ok(Permission {
        resource: resource.parse().map_err(|_| invalid())?,
        action: action.parse().map_err(|_| invalid())?,
    })
}

#[derive(Deserialize)]
struct PermissionsForm {
    #[serde(default)]
    granted: Vec<String>,
    #[serde(default)]
    permission: Vec<String>,
}

async fn set_permissions(
    State(state): State<AppState>,
    User(user): User,
    Path(IdParam { id }): Path<IdParam>,
    headers: HeaderMap,
    Form(form): Form<PermissionsForm>,
) -> Result<Redirect, UiError> {
    let parse_all = |values: &[String]| {
        values
            .iter()
            .map(|value| parse_permission(value))
            .collect::<Result<Vec<_>, _>>()
    };

    let granted = parse_all(&form.granted)?;
    let wanted = parse_all(&form.permission)?;

    let to_grant: Vec<_> = wanted
        .iter()
        .filter(|p| !granted.contains(p))
        .copied()
        .collect();
    let to_revoke: Vec<_> = granted
        .iter()
        .filter(|p| !wanted.contains(p))
        .copied()
        .collect();

    let Json(_) = state
        .in_transaction(user, async |tx| {
            grant_permissions(tx, id, &to_grant).await?;
            revoke_permissions(tx, id, &to_revoke).await
        })
        .await?;

    Ok(redirect_back(&headers))
}

#[derive(Deserialize)]
struct MembersForm {
    add: Option<Uuid>,
    remove: Option<Uuid>,
}

async fn asset(Path(file): Path<String>) -> Response {
    let (content_type, body): (&str, &[u8]) = match file.as_str() {
        "app.css" => ("text/css", include_bytes!("../assets/app.css")),
        "app.js" => ("text/javascript", include_bytes!("../assets/app.js")),
        "favicon.png" => ("image/png", include_bytes!("../assets/favicon.png")),
        _ => return StatusCode::NOT_FOUND.into_response(),
    };

    ([(header::CONTENT_TYPE, content_type)], body).into_response()
}
