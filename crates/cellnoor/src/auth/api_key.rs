use cellnoor_types::api_key::ApiKeyRecord;
use futures::StreamExt;
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

enum ApiKeyUserType {
    Person,
    Service,
}

trait ApiKeyExt {
    fn is_for_person(&self) -> bool;

    fn is_for_service(&self) -> bool;

    fn api_key_type(&self) -> Result<ApiKeyUserType, ErrorInner>;

    fn is_expired(&self) -> bool;

    fn to_user(&self) -> Result<AuthUser, ErrorInner>;
}

impl ApiKeyExt for ApiKeyRecord {
    fn is_for_person(&self) -> bool {
        self.person_id.is_some()
    }

    fn is_for_service(&self) -> bool {
        self.service_account_id.is_some()
    }

    // Technically, the db ensures we don't have both a `person_id` and
    // `service_account_id`, but it's easy to be certain of that at compile time
    fn api_key_type(&self) -> Result<ApiKeyUserType, ErrorInner> {
        let user_type = if self.is_for_person() && !self.is_for_service() {
            ApiKeyUserType::Person
        } else if !self.is_for_person() && self.is_for_service() {
            ApiKeyUserType::Service
        } else {
            return Err(ErrorInner::Other {
                message: format!(
                    "API key {} is assigned both to a person and a service account",
                    self.id
                ),
                sql_state: None,
            });
        };

        Ok(user_type)
    }

    fn is_expired(&self) -> bool {
        self.expires_at < jiff::Timestamp::now()
    }

    fn to_user(&self) -> Result<AuthUser, ErrorInner> {
        if self.is_expired() {
            return Err(ErrorInner::ExpiredApiKey {
                expired_at: self.expires_at,
            });
        }

        let user = match self.api_key_type()? {
            ApiKeyUserType::Person => AuthUser(DbUser::Person(self.id)),
            ApiKeyUserType::Service => AuthUser(DbUser::Service(self.id)),
        };

        Ok(user)
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

        let (_, api_key) = insert_test_api_key(&tx, AuthUser::new_as_admin(), |_| ())
            .await
            .unwrap();
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
