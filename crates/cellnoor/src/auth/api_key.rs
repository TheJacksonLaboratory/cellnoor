use cellnoor_types::api_key::SavedApiKeyRecord;
use sha3::Digest;

use crate::{auth::AuthUser, db, error::ErrorInner, state::AppState};

pub(super) async fn authenticate_with_api_key(
    state: &AppState,
    api_key: &[u8],
) -> Result<AuthUser, ErrorInner> {
    let api_key_record = fetch_api_key_record_by_hash(&state.db_pool, api_key).await?;

    AuthUser::from_api_key_record(&api_key_record)
}

impl AuthUser {
    fn from_api_key_record(api_key: &SavedApiKeyRecord) -> Result<Self, ErrorInner> {
        if api_key_is_expired(api_key) {
            return Err(ErrorInner::ExpiredApiKey {
                expired_at: api_key.expires_at.unwrap(),
            });
        }

        Ok(Self {
            id: api_key.owner_id,
            is_staff: api_key.owner_is_staff,
        })
    }
}

fn api_key_is_expired(api_key: &SavedApiKeyRecord) -> bool {
    api_key
        .expires_at
        .is_some_and(|e| e < jiff::Timestamp::now())
}

// We don't know who the user is until we've found their API key, so this can't
// go through `db::Client`. The SQL function runs as its owner and is the only
// thing that can read a hashed key
async fn fetch_api_key_record_by_hash(
    pool: &db::Pool,
    api_key: &[u8],
) -> Result<SavedApiKeyRecord, ErrorInner> {
    let hashed_key = hash_api_key(api_key);

    let row = pool
        .unauthenticated()
        .await?
        .query_one("select api_key_by_hash($1)", &[&hashed_key])
        .await?;

    Ok(row.get(0))
}

pub fn hash_api_key(api_key: &[u8]) -> [u8; 32] {
    let mut hasher = sha3::Sha3_256::new();

    hasher.update(api_key);
    hasher.finalize().0
}

#[cfg(test)]
mod tests {
    use crate::{
        auth::api_key::fetch_api_key_record_by_hash,
        handlers::api_keys::insert_test_api_key,
        state::test_util::{db_client_as_admin, test_db_pool},
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn fetch_by_hash() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, api_key) = insert_test_api_key(&tx, |_| ()).await.unwrap();
        tx.commit().await.unwrap();

        assert_eq!(
            fetch_api_key_record_by_hash(&test_db_pool(), api_key.secret.as_bytes())
                .await
                .unwrap(),
            api_key.record
        );
    }
}
