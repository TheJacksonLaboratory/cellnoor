use cellnoor_types::api_key::PersonId;
use jsonwebtoken::TokenData;
use uuid::Uuid;

use crate::{
    auth::{AuthUser, DbUser},
    error::ErrorInner,
};

pub(super) fn authenticate_with_jwt(
    token: &[u8],
    decoding_key: &jsonwebtoken::DecodingKey,
    validation: &jsonwebtoken::Validation,
) -> Result<AuthUser, ErrorInner> {
    jsonwebtoken::decode(token, decoding_key, validation)
        .map(AuthUser::from_token_data)
        .map_err(|e| ErrorInner::InvalidAuthToken {
            message: e.to_string(),
        })
}

impl AuthUser {
    fn from_token_data(token: TokenData<Claims>) -> Self {
        let UserClaims {
            id,
            can_read_all_projects,
        } = token.claims.user;
        AuthUser(DbUser::Jwt {
            user_id: id,
            can_read_all_projects,
        })
    }
}

#[allow(dead_code)]
#[derive(serde::Deserialize)]
struct Claims {
    user: UserClaims,
    iat: usize,
    exp: usize,
}

#[allow(dead_code)]
#[derive(serde::Deserialize)]
struct UserClaims {
    id: PersonId,
    can_read_all_projects: bool,
}
