use jsonwebtoken::TokenData;
use uuid::Uuid;

use crate::{auth::AuthUser, error::ErrorInner};

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
        let UserClaims { id, is_staff } = token.claims.user;
        AuthUser { id, is_staff }
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
    id: Uuid,
    is_staff: bool,
}
