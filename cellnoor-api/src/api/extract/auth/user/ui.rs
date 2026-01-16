use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
struct Claims {
    sub: Uuid,
    is_admin: bool,
    is_biology_staff: bool,
    is_computational_staff: bool,
    iat: usize,
    exp: usize,
    iss: String,
    aud: String,
}
