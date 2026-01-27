use std::borrow::Cow;

use aide::{
    openapi::{Content, MediaType, Parameter, ParameterSchemaOrContent, QueryStyle, SchemaObject},
    operation::{ParamLocation, add_parameters, parameters_from_schema},
};
use axum::extract::FromRequestParts;
use schemars::JsonSchema;
use serde::de::DeserializeOwned;

use crate::api;

#[derive(Default, JsonSchema)]
#[serde(default)]
#[schemars(inline)]
pub struct JsonQuery<T>
where
    T: Default,
{
    pub query: T,
}

impl<T> aide::OperationInput for JsonQuery<T>
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

impl<S, T> FromRequestParts<S> for JsonQuery<T>
where
    S: Sync,
    T: Default + DeserializeOwned,
{
    type Rejection = api::Error;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let Some(query) = parts.uri.query() else {
            return Ok(Self {
                query: T::default(),
            });
        };

        let mut parsed_querystring = form_urlencoded::parse(query.as_bytes());
        let Some((Cow::Borrowed("query"), s)) = parsed_querystring.next() else {
            return Err(api::Error::MalformedRequest {
                message: "failed to parse query string - missing parameter 'query'".to_owned(),
            });
        };

        let query =
            serde_json::from_slice(s.as_bytes()).map_err(|e| api::Error::MalformedRequest {
                message: format!("failed to parse JSON in query string: {e}"),
            })?;

        Ok(Self { query })
    }
}
