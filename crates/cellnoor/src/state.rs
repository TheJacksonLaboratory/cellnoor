use std::sync::Arc;

use camino::{Utf8Path, Utf8PathBuf};
use deadpool_postgres::PoolError;
use secrecy::ExposeSecret;

use crate::{auth::AuthUser, db, settings::Settings};

#[derive(Clone)]
struct StateCommon {
    db_pool: db::Pool,
    public_files_url: String,
    static_file_dir: Utf8PathBuf,
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
        &self.inner.public_files_url
    }
}

#[derive(Clone)]
pub struct ProdState {
    inner: StateCommon,
    jwt_decoding_info: Arc<(jsonwebtoken::DecodingKey, jsonwebtoken::Validation)>,
}

impl ProdState {
    pub async fn db_client(&self, user: AuthUser) -> Result<db::Client, PoolError> {
        self.inner.db_pool.get(user).await
    }

    pub fn jwt_decoding_info(&self) -> &(jsonwebtoken::DecodingKey, jsonwebtoken::Validation) {
        &self.jwt_decoding_info
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
            db_pool: db::Pool::new(settings.db_config().to_owned(), settings.max_db_pool_size())?,
            public_files_url: settings.public_files_url().to_owned(),
            static_file_dir: Utf8PathBuf::from(settings.static_file_dir()),
        };

        let state = if settings.with_auth() {
            Self::Prod(ProdState {
                inner,
                jwt_decoding_info: Arc::new((
                    jsonwebtoken::DecodingKey::from_secret(
                        settings.auth_secret().expose_secret().as_bytes(),
                    ),
                    jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256),
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

    pub fn public_files_url(&self) -> &str {
        &self.inner().public_files_url
    }

    pub fn static_file_dir(&self) -> &Utf8Path {
        &self.inner().static_file_dir
    }
}

// We put this module inside of `state.rs` so it has full access to `ProdState`
/// A module of test utilities to reduce boilerplate for writing tests.
#[cfg(test)]
pub mod test_util {
    use std::sync::Arc;

    use camino::Utf8PathBuf;
    
    use nonempty::NonemptyString;
    #[cfg(test)]
    use uuid::Uuid;

    use crate::{
        auth::AuthUser,
        db,
        state::{ProdState, StateCommon},
    };

    fn new_test_state() -> ProdState {
        use std::env;

        // This looks like it won't compile but it will when you run
        // ./scripts/dev/test.sh
        let db_pool = db::Pool::from_url(env!("CELLNOOR_TEST_DB_URL"));

        // Unit-tests don't use JSON web tokens, so we pass in an empty secret
        ProdState {
            inner: StateCommon {
                db_pool,
                public_files_url: String::new(),
                static_file_dir: Utf8PathBuf::new(),
            },

            jwt_decoding_info: Arc::new((
                jsonwebtoken::DecodingKey::from_secret(&[]),
                // better-auth uses HS256 by default
                jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256),
            )),
        }
    }

    pub async fn db_client_as_app() -> db::Client {
        new_test_state()
            .db_client(AuthUser::new_as_app())
            .await
            .unwrap()
    }

    pub async fn db_client_as_user(user: Uuid) -> db::Client {
        new_test_state()
            .db_client(AuthUser::new_as_user(user))
            .await
            .unwrap()
    }

    pub async fn db_client_as_admin() -> db::Client {
        new_test_state()
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
