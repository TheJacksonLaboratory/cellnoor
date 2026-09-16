use camino::{Utf8Path, Utf8PathBuf};
use deadpool_postgres::PoolError;
use secrecy::ExposeSecret;

use crate::{auth::AuthUser, db, settings::Settings};

type JwtDecodingInfo = (jsonwebtoken::DecodingKey, jsonwebtoken::Validation);

// In theory, we shouldn't really be cloning strings on every request. I don't
// think this is a serious performance issue for like 100 bytes though
#[derive(Clone)]
pub struct AppState {
    db_pool: db::Pool,
    public_files_url: String,
    public_auth_url: String,
    static_files_dir: Utf8PathBuf,
    // `None` disables authentication: every request then runs as the admin user
    jwt_decoding_info: Option<&'static JwtDecodingInfo>,
}

impl AppState {
    pub fn initialize(settings: &Settings) -> anyhow::Result<Self> {
        let jwt_decoding_info = settings.with_auth().then(|| {
            &*Box::leak(Box::new((
                jsonwebtoken::DecodingKey::from_secret(
                    settings.auth_secret().expose_secret().as_bytes(),
                ),
                jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256),
            )))
        });

        Ok(Self {
            db_pool: db::Pool::new(settings.db_config().to_owned(), settings.max_db_pool_size())?,
            public_files_url: settings.public_files_url().to_owned(),
            public_auth_url: settings.public_auth_url().to_owned(),
            static_files_dir: Utf8PathBuf::from(settings.static_files_dir()),
            jwt_decoding_info,
        })
    }

    pub async fn db_client(&self, user: AuthUser) -> Result<db::Client, PoolError> {
        self.db_pool.get(user).await
    }

    pub fn db_pool(&self) -> &db::Pool {
        &self.db_pool
    }

    pub fn jwt_decoding_info(&self) -> Option<&'static JwtDecodingInfo> {
        self.jwt_decoding_info
    }

    pub fn public_files_url(&self) -> &str {
        &self.public_files_url
    }

    pub fn static_files_dir(&self) -> &Utf8Path {
        &self.static_files_dir
    }

    pub fn public_auth_url(&self) -> &str {
        &self.public_auth_url
    }
}

/// A module of test utilities to reduce boilerplate for writing tests.
#[cfg(test)]
pub mod test_util {
    use nonempty::NonemptyString;
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
