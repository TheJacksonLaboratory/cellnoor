use cellnoor_types::api_key::ApiKeyRecord;
use sha3::Digest;

use crate::{
    auth::{AuthUser, DbUser},
    db::{self, SqlBuilder},
    error::ErrorInner,
    state::ProdState,
};

pub(super) async fn authenticate_with_api_key(
    state: &ProdState,
    api_key: &[u8],
) -> Result<AuthUser, ErrorInner> {
    let mut client = state.db_client(AuthUser(DbUser::App)).await?;
    let tx = client.begin().await?;

    let api_key_record = fetch_api_key_record_by_hash(&tx, api_key).await?;

    api_key_record.to_user()
}

trait ApiKeyExt {
    fn is_expired(&self) -> bool;

    fn to_user(&self) -> Result<AuthUser, ErrorInner>;
}

impl ApiKeyExt for ApiKeyRecord {
    fn is_expired(&self) -> bool {
        self.expires_at.is_some_and(|e| e < jiff::Timestamp::now())
    }

    fn to_user(&self) -> Result<AuthUser, ErrorInner> {
        if self.is_expired() {
            return Err(ErrorInner::ExpiredApiKey {
                expired_at: self.expires_at.unwrap(),
            });
        }

        let user = match (self.person_id, self.service_id) {
            (Some(id), None) => DbUser::PersonApiKey(id),
            (None, Some(id)) => DbUser::ServiceApiKey(id),
            _ => {
                return Err(ErrorInner::Other {
                    message: "API key belongs to both person and service".to_owned(),
                    sql_state: None,
                });
            }
        };

        Ok(AuthUser(user))
    }
}

async fn fetch_api_key_record_by_hash(
    tx: &db::Transaction<'_>,
    api_key: &[u8],
) -> Result<ApiKeyRecord, ErrorInner> {
    static SELECT_API_KEY: SqlBuilder = SqlBuilder::new(include_str!("select_api_key.sql"));

    let hashed_key = hash_api_key(api_key);

    let sql = SELECT_API_KEY.finish_with_params(vec![&hashed_key]);

    let api_key_record = tx.query_one_into(&sql).await?;

    Ok(api_key_record)
}

pub fn hash_api_key(api_key: &[u8]) -> [u8; 32] {
    let mut hasher = sha3::Sha3_256::new();

    hasher.update(api_key);
    hasher.finalize().0
}

#[cfg(test)]
mod tests {
    use crate::{
        auth::{AuthUser, api_key::fetch_api_key_record_by_hash},
        handlers::api_keys::insert_test_api_key,
        state::test_util::{db_client_as_admin, db_client_as_app},
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn fetch_by_hash() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, api_key) = insert_test_api_key(&tx, |_| ()).await.unwrap();
        tx.commit().await.unwrap();

        let mut client = db_client_as_app().await;
        let tx = client.begin().await.unwrap();

        assert_eq!(
            fetch_api_key_record_by_hash(&tx, api_key.secret.as_bytes())
                .await
                .unwrap(),
            api_key.record
        );
    }
}
