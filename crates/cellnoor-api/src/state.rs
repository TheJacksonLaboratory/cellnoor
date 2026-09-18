use axum::Json;
use camino::Utf8PathBuf;
use deadpool_postgres::PoolError;
use secrecy::ExposeSecret;

use crate::{
    auth::AuthUser,
    db::{self, DbError},
    settings::Settings,
};

type JwtDecodingInfo = (jsonwebtoken::DecodingKey, jsonwebtoken::Validation);

// In theory, we shouldn't really be cloning strings on every request. I don't
// think this is a serious performance issue for like 100 bytes though
#[derive(Clone)]
pub struct AppState {
    pub db_pool: db::Pool,
    pub static_files_dir: Utf8PathBuf,
    // `None` disables authentication: every request then runs as the admin user
    pub jwt_decoding_info: Option<&'static JwtDecodingInfo>,
}

impl AppState {
    pub fn initialize(settings: &Settings) -> anyhow::Result<Self> {
        let jwt_decoding_info = settings.with_auth.then(|| {
            &*Box::leak(Box::new((
                jsonwebtoken::DecodingKey::from_secret(
                    settings.auth_secret.expose_secret().as_bytes(),
                ),
                jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256),
            )))
        });

        Ok(Self {
            db_pool: db::Pool::new(settings.db.clone(), settings.max_db_pool_size)?,
            static_files_dir: Utf8PathBuf::from(&settings.static_files_dir),
            jwt_decoding_info,
        })
    }

    pub async fn db_client(&self, user: AuthUser) -> Result<db::Client, PoolError> {
        self.db_pool.get(user).await
    }

    /// Run `work` in one transaction on behalf of `user`, committing it only if
    /// `work` succeeds.
    pub async fn in_transaction<T, E>(
        &self,
        user: AuthUser,
        work: impl AsyncFnOnce(&db::Transaction<'_>) -> Result<T, E>,
    ) -> Result<Json<T>, E>
    where
        E: From<DbError>,
    {
        let mut client = self.db_client(user).await.map_err(DbError::from)?;
        let tx = client.begin().await.map_err(DbError::from)?;

        let response = work(&tx).await?;

        tx.commit().await.map_err(DbError::from)?;

        Ok(Json(response))
    }
}

#[cfg(test)]
pub mod test_util {
    use cellnoor_types::nonempty::NonemptyString;
    use uuid::Uuid;

    use crate::{auth::AuthUser, db};

    pub fn test_db_pool() -> db::Pool {
        // This looks like it won't compile but it will when you run
        // ./scripts/dev/test.sh
        db::Pool::from_url(env!("CELLNOOR_TEST_DB_URL"))
    }

    pub async fn db_client_as_admin() -> db::Client {
        test_db_pool().get(AuthUser::admin()).await.unwrap()
    }

    pub async fn db_client_as_user(user: Uuid) -> db::Client {
        test_db_pool()
            .get(AuthUser::new_as_user(user))
            .await
            .unwrap()
    }

    pub trait ToNonemptyString {
        fn to_nonempty_string(&self) -> NonemptyString;
    }

    impl<T> ToNonemptyString for T
    where
        T: AsRef<str>,
    {
        fn to_nonempty_string(&self) -> NonemptyString {
            NonemptyString::new(self.as_ref().to_owned()).unwrap()
        }
    }
}
