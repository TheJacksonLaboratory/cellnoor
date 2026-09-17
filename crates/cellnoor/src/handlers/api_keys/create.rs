use axum::{Json, extract::State};
use cellnoor_types::{
    Relation,
    api_key::{ApiKey, NewApiKey},
    nonempty::NonemptyString,
};
use jiff::Timestamp;
use rand::{RngExt, distr::Alphanumeric};
use uuid::Uuid;

use crate::{
    auth::{AuthUser, hash_api_key},
    db::{self, FieldValues, Insert},
    error::{Error, ErrorInner},
    handlers::api_keys::index::select_api_key_record_by_id,
    state::AppState,
};

pub async fn create_api_key(
    State(state): State<AppState>,
    user: AuthUser,
    new_api_key: Option<Json<NewApiKey>>,
) -> Result<Json<ApiKey>, Error> {
    state
        .in_transaction(user, async |tx| {
            insert_api_key(tx, &new_api_key.unwrap_or_default()).await
        })
        .await
}

async fn insert_api_key(
    tx: &db::Transaction<'_>,
    NewApiKey {
        description,
        owner_id,
        expires_at,
    }: &NewApiKey,
) -> Result<ApiKey, ErrorInner> {
    let secret = generate_secret();

    let record = NewApiKeyRecord {
        description: description.as_ref(),
        hashed_key: hash_api_key(secret.as_bytes()),
        owner_id: owner_id.unwrap_or(tx.user().id()),
        expires_at: *expires_at,
    };

    let id = tx.insert_returning_id(&record).await?;

    let record = select_api_key_record_by_id(tx, id).await?;

    Ok(ApiKey { record, secret })
}

struct NewApiKeyRecord<'a> {
    description: Option<&'a NonemptyString>,
    hashed_key: [u8; 32],
    owner_id: Uuid,
    expires_at: Option<Timestamp>,
}

impl Relation for NewApiKeyRecord<'_> {
    const NAME: &'static str = "api_key";
}

impl Insert for NewApiKeyRecord<'_> {
    type Field = &'static str;

    fn fields(&self) -> FieldValues<'_, Self::Field> {
        let Self {
            description,
            hashed_key,
            owner_id,
            expires_at,
        } = self;

        vec![
            ("description", description),
            ("hashed_key", hashed_key),
            ("owner_id", owner_id),
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
            owner_id: None,
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
