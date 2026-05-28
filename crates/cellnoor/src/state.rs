use std::sync::Arc;

use deadpool_postgres::PoolError;
use secrecy::ExposeSecret;

use crate::{auth::AuthUser, db, settings::Settings};

#[derive(Clone)]
struct StateCommon {
    db_pool: db::Pool,
    raw_files_url: String,
}

#[derive(Clone)]
pub struct DevState {
    inner: StateCommon,
}

impl DevState {
    pub fn db_pool(&self) -> db::Pool {
        self.inner.db_pool.clone()
    }

    pub fn raw_files_url(&self) -> &str {
        &self.inner.raw_files_url
    }
}

#[derive(Clone)]
pub struct ProdState {
    inner: StateCommon,
    jwt_decoding_key: Arc<jsonwebtoken::DecodingKey>,
}

impl ProdState {
    pub async fn db_client(&self, user: AuthUser) -> Result<db::Client, PoolError> {
        self.inner.db_pool.get(user).await
    }

    pub fn jwt_decoding_key(&self) -> &jsonwebtoken::DecodingKey {
        &self.jwt_decoding_key
    }
}

#[derive(Clone)]
pub enum AppState {
    Dev(DevState),
    Prod(ProdState),
}

impl AppState {
    pub fn initialize(settings: &Settings) -> anyhow::Result<Self> {
        let inner = StateCommon {
            db_pool: db::Pool::new(
                settings.db_url().expose_secret(),
                settings.max_db_pool_size(),
            )?,
            raw_files_url: settings.raw_files_url().to_owned(),
        };

        let state = if settings.with_auth() {
            Self::Prod(ProdState {
                inner,
                jwt_decoding_key: Arc::new(jsonwebtoken::DecodingKey::from_secret(
                    settings.auth_secret().expose_secret().as_bytes(),
                )),
            })
        } else {
            Self::Dev(DevState { inner })
        };

        Ok(state)
    }

    pub async fn db_client(
        &self,
        user: AuthUser,
    ) -> Result<db::Client, deadpool_postgres::PoolError> {
        match self {
            Self::Dev(s) => s.db_client().await,
            Self::Prod(s) => s.db_client(user).await,
        }
    }

    fn inner(&self) -> &StateCommon {
        match self {
            Self::Dev(DevState { inner }) | Self::Prod(ProdState { inner, .. }) => inner,
        }
    }

    pub fn raw_files_url(&self) -> &str {
        &self.inner().raw_files_url
    }
}

// We put this module inside of `state.rs` so it has full access to `ProdState`
/// A module of test utilities to reduce boilerplate for writing tests.
#[cfg(test)]
pub mod test_util {
    use std::sync::Arc;

    use nonempty::NonemptyString;
    #[cfg(test)]
    use uuid::Uuid;

    use crate::{
        auth::AuthUser,
        db,
        state::{ProdState, StateCommon},
    };

    fn test_state() -> ProdState {
        use std::env;

        dotenvy::dotenv().ok();

        let db_pool = db::Pool::new(
            &env::var("CELLNOOR_TEST_DB_URL")
                .expect("environment variables 'CELLNOOR_TEST_DB_URL' required for test"),
            None,
        )
        .unwrap();

        // Unit-tests don't use JSON web tokens, so we pass in an empty secret
        ProdState {
            inner: StateCommon {
                db_pool,
                raw_files_url: String::new(),
            },

            jwt_decoding_key: Arc::new(jsonwebtoken::DecodingKey::from_secret(&[])),
        }
    }

    static TEST_STATE: std::sync::LazyLock<ProdState> = std::sync::LazyLock::new(test_state);

    pub async fn db_client_as_app() -> db::Client {
        TEST_STATE.db_client(AuthUser::new_as_app()).await.unwrap()
    }

    pub async fn db_client_as_user(user: Uuid) -> db::Client {
        TEST_STATE
            .db_client(AuthUser::new_as_user(user))
            .await
            .unwrap()
    }

    pub async fn db_client_as_admin() -> db::Client {
        TEST_STATE
            .db_client(AuthUser::new_as_admin())
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
