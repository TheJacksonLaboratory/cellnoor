use axum::{Json, extract::State};
use cellnoor_types::{
    api_key::{ApiKey, NewApiKey},
    person::PermissionsToRevoke,
};
use jiff::Timestamp;
use nonempty::NonemptyString;
use rand::{RngExt, distr::Alphanumeric};
use uuid::Uuid;

use crate::{
    auth::{AuthUser, hash_api_key},
    db::{self, AsFieldValuePairs, FieldValuePairs},
    error::{Error, ErrorInner},
    handlers::{api_keys::index::select_api_key_record_by_id, people::create::provision_db_user},
    state::AppState,
};

pub async fn create_api_key(
    State(state): State<AppState>,
    user: AuthUser,
    Json(new_api_key): Json<NewApiKey>,
) -> Result<Json<ApiKey>, Error> {
    let mut client = state.db_client(user).await?;

    let tx = client.begin().await?;

    let response = insert_api_key(&tx, user, &new_api_key).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

async fn insert_api_key(
    tx: &db::Transaction<'_>,
    user: AuthUser,
    NewApiKey {
        description,
        service_account_id,
        expires_at,
        permissions_to_grant: grant_permissions,
    }: &NewApiKey,
) -> Result<ApiKey, ErrorInner> {
    let secret = generate_secret();

    let person_id = match service_account_id {
        Some(_) => None,
        None => Some(user.id().ok_or(ErrorInner::Other {
            message: "something went wrong".to_owned(),
            sql_state: None,
        })?),
    };

    let record = NewApiKeyRecord {
        description: description.as_ref(),
        hashed_key: hash_api_key(secret.as_bytes()),
        person_id,
        service_account_id: *service_account_id,
        expires_at: *expires_at,
    };

    let id = db::insert_into(tx, "api_key", &record).await?;

    let revoke_permissions = PermissionsToRevoke::default();
    let (_, record) = tokio::try_join!(
        provision_db_user(tx, id, grant_permissions, &revoke_permissions),
        select_api_key_record_by_id(tx, id),
    )?;

    Ok(ApiKey { record, secret })
}

struct NewApiKeyRecord<'a> {
    description: Option<&'a NonemptyString>,
    hashed_key: [u8; 32],
    person_id: Option<Uuid>,
    service_account_id: Option<Uuid>,
    expires_at: Timestamp,
}

impl AsFieldValuePairs<&'static str, 5> for NewApiKeyRecord<'_> {
    fn as_field_value_pairs(&self) -> FieldValuePairs<'_, &'static str, 5> {
        let Self {
            description,
            hashed_key,
            person_id,
            service_account_id,
            expires_at,
        } = self;

        [
            ("description", description),
            ("hashed_key", hashed_key),
            ("person_id", person_id),
            ("service_account_id", service_account_id),
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
        user: AuthUser,
        mut modify: F,
    ) -> Result<(NewApiKey, ApiKey), ErrorInner>
    where
        F: FnMut(&mut NewApiKey),
    {
        let mut new = NewApiKey {
            description: Some(Uuid::new_v4().to_string().to_nonempty_string()),
            service_account_id: None,
            expires_at: "2999-12-31T23:59:59Z".parse().unwrap(),
            permissions_to_grant: vec![].into(),
        };

        modify(&mut new);

        let inserted = insert_api_key(tx, user, &new).await?;
        Ok((new, inserted))
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        insert_test_api_key(&tx, AuthUser::new_as_admin(), |_| ())
            .await
            .unwrap();
    }
}
