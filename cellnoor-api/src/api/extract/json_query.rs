use std::borrow::Cow;

use aide::{
    openapi::{Content, MediaType, ParameterSchemaOrContent, SchemaObject},
    operation::{ParamLocation, add_parameters, parameters_from_schema},
};
use axum::{
    Extension, Json, RequestPartsExt, extract::FromRequestParts, http::StatusCode,
    response::IntoResponse,
};
use schemars::JsonSchema;
use serde::{Serialize, de::DeserializeOwned};

use crate::{
    api::auth::{self, AuthUser},
    state::AppState,
};

#[derive(Default, JsonSchema)]
#[serde(default)]
#[schemars(inline)]
pub struct AuthJsonQuery<T>
where
    T: Default,
{
    pub q: T,
}

impl<T> aide::OperationInput for AuthJsonQuery<T>
where
    T: Default,
    T: JsonSchema,
{
    fn operation_input(
        ctx: &mut aide::generate::GenContext,
        operation: &mut aide::openapi::Operation,
    ) {
        let schema = ctx.schema.subschema_for::<Self>();
        let mut params = parameters_from_schema(ctx, schema, ParamLocation::Query);

        let parameter = params
            .get_mut(0)
            .expect("there should be one parameter called query");

        parameter.parameter_data_mut().format =
            ParameterSchemaOrContent::Content(Content::from([(
                "application/json".to_owned(),
                MediaType {
                    schema: Some(SchemaObject {
                        json_schema: ctx.schema.subschema_for::<T>(),
                        example: None,
                        external_docs: None,
                    }),

                    ..Default::default()
                },
            )]));

        add_parameters(ctx, operation, params);
    }
}

#[derive(Debug, thiserror::Error, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type")]
#[schemars(rename = "ParseJsonQueryError")]
pub enum Error {
    #[error(transparent)]
    Auth(#[from] auth::Error),
    #[error("query-string is missing parameter {missing_parameter}")]
    MissingParameter { missing_parameter: &'static str },
    #[error("{message}")]
    ParseJson { message: String },
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::UNPROCESSABLE_ENTITY, Json(self)).into_response()
    }
}

pub trait Authorize: Sized {
    fn authorize(self, user: &AuthUser) -> Result<Self, auth::Error>;
}

impl<T> FromRequestParts<AppState> for AuthJsonQuery<T>
where
    T: Default + DeserializeOwned + Authorize,
{
    type Rejection = Error;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let user = match parts.extract_with_state(state).await {
            Ok(Extension(user)) => user,
            Err(_) => {
                AuthUser::from_request(
                    state,
                    parts.extract::<Option<_>>().await.unwrap().as_ref(),
                    &parts.extract().await.unwrap(),
                )
                .await?
            }
        };

        let Some(q) = parts.uri.query() else {
            return Ok(Self {
                q: T::default().authorize(&user)?,
            });
        };

        let mut parsed_querystring = form_urlencoded::parse(q.as_bytes());
        let Some((Cow::Borrowed("q"), s)) = parsed_querystring.next() else {
            return Err(Error::MissingParameter {
                missing_parameter: "q",
            });
        };

        let q: T = serde_json::from_slice(s.as_bytes()).map_err(|e| Error::ParseJson {
            message: format!("failed to parse JSON in query string: {e}"),
        })?;

        Ok(Self {
            q: q.authorize(&user)?,
        })
    }
}
