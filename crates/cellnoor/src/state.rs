use std::sync::Arc;

use deadpool_postgres::PoolError;
use secrecy::ExposeSecret;
use uuid::Uuid;

use crate::{
    db::{self},
    settings::Settings,
};

#[derive(Clone)]
pub struct DevState {
    db_pool: db::Pool,
}
impl DevState {
    pub async fn db_client(&self) -> Result<db::Client, PoolError> {
        self.db_pool.get(db::User::Person(Uuid::nil())).await
    }
}

#[derive(Clone)]
pub struct ProdState {
    db_pool: db::Pool,
    // Store these two things in one `Arc` instead of 2
    jwt_decoding_key: Arc<jsonwebtoken::DecodingKey>,
}

impl ProdState {
    pub async fn db_client(&self, user: db::User) -> Result<db::Client, PoolError> {
        self.db_pool.get(user).await
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
    pub fn initialize(settings: Settings) -> anyhow::Result<Self> {
        let db_pool = db::Pool::new(
            settings.db_url().expose_secret(),
            settings.max_db_pool_size(),
        )?;

        let state = if settings.with_auth() {
            Self::Prod(ProdState {
                db_pool,
                jwt_decoding_key: Arc::new(jsonwebtoken::DecodingKey::from_secret(
                    settings.auth_secret().expose_secret().as_bytes(),
                )),
            })
        } else {
            Self::Dev(DevState { db_pool })
        };

        Ok(state)
    }

    pub async fn db_client(
        &self,
        user: db::User,
    ) -> Result<db::Client, deadpool_postgres::PoolError> {
        match self {
            Self::Dev(s) => s.db_client().await,
            Self::Prod(s) => s.db_client(user).await,
        }
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

    use crate::{db, state::ProdState};

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
            db_pool,
            jwt_decoding_key: Arc::new(jsonwebtoken::DecodingKey::from_secret(&[])),
        }
    }

    static TEST_STATE: std::sync::LazyLock<ProdState> = std::sync::LazyLock::new(test_state);

    #[cfg(test)]
    pub async fn db_client_as_app() -> db::Client {
        TEST_STATE.db_client(db::User::App).await.unwrap()
    }

    #[cfg(test)]
    pub async fn db_client_as_user(user: Uuid) -> db::Client {
        TEST_STATE.db_client(db::User::Person(user)).await.unwrap()
    }

    #[cfg(test)]
    pub async fn db_client_as_admin() -> db::Client {
        TEST_STATE
            .db_client(db::User::Person(Uuid::nil()))
            .await
            .unwrap()
    }

    #[cfg(test)]
    pub async fn create_user() -> Uuid {
        let client = db_client_as_admin().await;

        todo!()
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
