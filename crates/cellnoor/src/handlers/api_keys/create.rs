use axum::{Json, extract::State};
use cellnoor_types::{
    api_key::{ApiKey, NewApiKey, PersonId, ServiceId},
    person::{PermissionsToGrant, PermissionsToRevoke},
};
use jiff::Timestamp;
use nonempty::NonemptyString;
use rand::{RngExt, distr::Alphanumeric};
use uuid::Uuid;

use crate::{
    auth::{AuthUser, hash_api_key},
    db::{self, AsFieldValuePairs, FieldValuePairs},
    error::{Error, ErrorInner},
    handlers::api_keys::index::select_api_key_record_by_id,
    state::AppState,
};

pub async fn create_api_key(
    State(state): State<AppState>,
    user: AuthUser,
    Json(new_api_key): Json<NewApiKey>,
) -> Result<Json<ApiKey>, Error> {
    let mut client = state.db_client(user).await?;

    let tx = client.begin().await?;

    let response = insert_api_key(&tx, &new_api_key).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

async fn insert_api_key(
    tx: &db::Transaction<'_>,
    NewApiKey {
        description,
        service_id,
        expires_at,
    }: &NewApiKey,
) -> Result<ApiKey, ErrorInner> {
    let secret = generate_secret();

    // Note: we don't use tx.user().service_id() because firstly, it would be
    // impossible for a service to create an API key for itself without an API key,
    // and secondly, only people can create API keys
    let (person_id, service_id) = match service_id {
        Some(service_id) => (None, Some(*service_id)),
        None => (tx.user().person_id(), None),
    };

    let record = NewApiKeyRecord {
        description: description.as_ref(),
        hashed_key: hash_api_key(secret.as_bytes()),
        person_id,
        service_id,
        expires_at: *expires_at,
    };

    let id = db::insert_into(tx, "api_key", &record).await?;

    let record = select_api_key_record_by_id(tx, id).await?;

    Ok(ApiKey { record, secret })
}

struct NewApiKeyRecord<'a> {
    description: Option<&'a NonemptyString>,
    hashed_key: [u8; 32],
    person_id: Option<PersonId>,
    service_id: Option<ServiceId>,
    expires_at: Option<Timestamp>,
}

impl AsFieldValuePairs<&'static str, 5> for NewApiKeyRecord<'_> {
    fn as_field_value_pairs(&self) -> FieldValuePairs<'_, &'static str, 5> {
        let Self {
            description,
            hashed_key,
            person_id,
            service_id,
            expires_at,
        } = self;

        [
            ("description", description),
            ("hashed_key", hashed_key),
            ("person_id", person_id),
            ("service_id", service_id),
            ("expires_at", expires_at),
        ]
    }
}

fn generate_secret() -> String {
    // This gets 128 bits of entropy
    const SECRET_LEN: usize = 22;
    static PREFIX: &str = "cellnoor_";

    let rng = rand::rng();

    let mut secret = String::with_capacity(SECRET_LEN + PREFIX.len());
    secret.push_str(PREFIX);

    for c in rng
        .sample_iter(Alphanumeric)
        .take(SECRET_LEN)
        .map(char::from)
    {
        secret.push(c);
    }

    secret
}

#[cfg(test)]
pub mod test {
    use cellnoor_types::api_key::{ApiKey, NewApiKey};
    use uuid::Uuid;

    use crate::{
        auth::AuthUser,
        db,
        error::ErrorInner,
        handlers::api_keys::create::insert_api_key,
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    pub async fn insert_test_api_key<F>(
        tx: &db::Transaction<'_>,
        mut modify: F,
    ) -> Result<(NewApiKey, ApiKey), ErrorInner>
    where
        F: FnMut(&mut NewApiKey),
    {
        let mut new = NewApiKey {
            description: Some(Uuid::new_v4().to_string().to_nonempty_string()),
            service_id: None,
            expires_at: None,
        };

        modify(&mut new);

        let inserted = insert_api_key(tx, &new).await?;
        Ok((new, inserted))
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        insert_test_api_key(&tx, |_| ()).await.unwrap();
    }
}
