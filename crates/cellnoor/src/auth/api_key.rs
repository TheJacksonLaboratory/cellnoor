use sha3::Digest;
use uuid::Uuid;

use crate::{auth::AuthUser, db, error::Error};

const API_KEY_LENGTH: usize = 22;

pub struct ApiKeyRecord {
    id: Uuid,
    person_id: Option<Uuid>,
    service_account_id: Option<Uuid>,
    expires_at: jiff::Timestamp,
}

enum ApiKeyUserType {
    Person,
    Service,
}

impl ApiKeyRecord {
    fn is_for_person(&self) -> bool {
        self.person_id.is_some()
    }

    fn is_for_service(&self) -> bool {
        self.service_account_id.is_some()
    }

    // Technically, the db ensures we don't have both a `person_id` and
    // `service_account_id`, but it's easy to be certain of that at compile time
    fn api_key_type(&self) -> Result<ApiKeyUserType, Error> {
        let user_type = if self.is_for_person() && !self.is_for_service() {
            ApiKeyUserType::Person
        } else if !self.is_for_person() && self.is_for_service() {
            ApiKeyUserType::Service
        } else {
            return Err(Error::other(format!(
                "API key {} is assigned both to a person and a service account",
                self.id
            )));
        };

        Ok(user_type)
    }

    fn is_expired(&self) -> bool {
        self.expires_at < jiff::Timestamp::now()
    }

    pub fn to_user(&self) -> Result<AuthUser, Error> {
        if self.is_expired() {
            return Err(Error::expired_api_key(self.expires_at));
        }

        let user = match self.api_key_type()? {
            ApiKeyUserType::Person => AuthUser::Person(self.id),
            ApiKeyUserType::Service => AuthUser::Service(self.id),
        };

        Ok(user)
    }
}

fn hash_api_key(api_key: &[u8]) -> [u8; 32] {
    let mut hasher = sha3::Sha3_256::new();

    hasher.update(api_key);
    hasher.finalize().0
}

pub async fn fetch_api_key_record(
    api_key: &[u8],
    mut db_client: db::Client,
) -> Result<ApiKeyRecord, Error> {
    if api_key.len() != API_KEY_LENGTH {
        return Err(Error::invalid_api_key());
    }

    let hashed_key = hash_api_key(api_key);

    let tx = db_client.begin().await?;

    // We manually map the fields from the database record into the struct we return
    let hashed_api_key = tx
        .query_one(
            "select (id, person_id, service_account_id, expires_at) from api_key where hashed_key \
             = $1",
            &[&hashed_key],
        )
        .await
        .map(|r| ApiKeyRecord {
            id: r.get("id"),
            person_id: r.get("person_id"),
            service_account_id: r.get("service_account_id"),
            expires_at: r.get("expired_at"),
        })?;

    Ok(hashed_api_key)
}
