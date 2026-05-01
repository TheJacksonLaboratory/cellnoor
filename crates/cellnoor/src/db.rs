use std::fmt::Display;

use aide::OperationIo;
use deadpool_postgres::{
    GenericClient, Object as InnerClient, Pool as InnerPool, PoolError,
    Transaction as InnerTransaction,
    tokio_postgres::{Error as TokioPgError, Row, types::ToSql},
};
use postgres_types::FromSqlOwned;
use uuid::Uuid;

use crate::error::Error;

pub mod institution;

#[derive(Debug, Clone)]
pub struct Pool(InnerPool);

/// An authenticated database user.
///
/// This could represent a person using the UI, a person using the RESTful API,
/// a service account using the RESTful API, or the app itself switching into
/// one of the aforementioned users.
#[derive(Debug, Clone, Copy, PartialEq, Eq, OperationIo)]
pub enum User {
    App,
    Person(Uuid),
    Service(Uuid),
}

impl Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::App => "app".fmt(f),
            Self::Service(id) | Self::Person(id) => id.fmt(f),
        }
    }
}

impl Pool {
    // Use `anyhow::Result` because we only make the pool once at app-startup
    pub fn new(db_url: &str, max_size: Option<usize>) -> anyhow::Result<Self> {
        let mut cfg = deadpool_postgres::Config::new();
        cfg.url = db_url.to_owned().into();

        let mut builder = cfg.builder(deadpool_postgres::tokio_postgres::NoTls)?;
        if let Some(max_size) = max_size {
            builder = builder.max_size(max_size);
        }

        Ok(builder.build().map(Self)?)
    }

    pub async fn get(&self, user: User) -> Result<Client, PoolError> {
        Ok(Client {
            user,
            inner: self.0.get().await?,
        })
    }
}

/// A database client that wraps a transaction.
///
/// After any operations, the transaction must be committed by calling
/// [Client::commit], or else nothing will be saved to the database.
#[derive(Debug)]
pub struct Client {
    user: User,
    inner: InnerClient,
}

impl Client {
    pub async fn begin(&'_ mut self) -> Result<Transaction<'_>, TokioPgError> {
        let Self { user, inner } = self;

        Ok(Transaction {
            user: *user,
            inner: inner.transaction().await?,
        })
    }
}

pub struct Transaction<'a> {
    user: User,
    inner: InnerTransaction<'a>,
}

impl<'a> Transaction<'a> {
    async fn execute_as_user<T>(
        &self,
        operation: impl Future<Output = Result<T, TokioPgError>>,
    ) -> Result<T, TokioPgError> {
        let Self { user, inner } = self;

        let set_role = format!(r#"set local role "{user}" "#);
        let set_role = inner.execute(&set_role, &[]);

        let (_, result) = tokio::try_join!(set_role, operation)?;

        Ok(result)
    }

    pub async fn query(
        &self,
        statement: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Vec<Row>, TokioPgError> {
        self.execute_as_user(self.inner.query(statement, params))
            .await
    }

    pub async fn query_one(
        &self,
        statement: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Row, TokioPgError> {
        self.execute_as_user(self.inner.query_one(statement, params))
            .await
    }

    pub async fn query_one_scalar<T>(
        &self,
        statement: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<T, Error>
    where
        T: FromSqlOwned,
    {
        let row = self.query_one(statement, params).await?;

        if row.len() != 1 {
            return Err(Error::other(
                "query returned more than one column".to_owned(),
            ));
        }

        Ok(row.get(0))
    }

    pub async fn execute(
        &self,
        statement: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<u64, TokioPgError> {
        self.execute_as_user(self.inner.execute(statement, params))
            .await
    }

    pub async fn commit(self) -> Result<(), TokioPgError> {
        self.inner.commit().await
    }
}
