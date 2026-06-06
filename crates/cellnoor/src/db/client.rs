use cellnoor_types::person::ResourcePermission;
use deadpool_postgres::{
    GenericClient, Object as InnerClient, Pool as InnerPool, PoolError,
    Transaction as InnerTransaction,
    tokio_postgres::{Error as TokioPgError, Row, RowStream, types::ToSql},
};
use futures::{Stream, StreamExt};
use postgres_types::FromSqlOwned;
use uuid::Uuid;

use crate::{auth::AuthUser, db::Sql, error::ErrorInner};

#[derive(Debug, Clone)]
pub struct Pool(InnerPool);

impl Pool {
    // Use `anyhow::Result` because we only make the pool once at app-startup
    pub fn new(cfg: deadpool_postgres::Config, max_size: Option<usize>) -> anyhow::Result<Self> {
        let mut builder = cfg.builder(deadpool_postgres::tokio_postgres::NoTls)?;
        if let Some(max_size) = max_size {
            builder = builder.max_size(max_size);
        }

        Ok(builder.build().map(Self)?)
    }

    #[cfg(test)]
    pub fn from_url(db_url: &str) -> Self {
        let mut cfg = deadpool_postgres::Config::new();

        cfg.url = db_url.to_owned().into();
        let builder = cfg
            .builder(deadpool_postgres::tokio_postgres::NoTls)
            .unwrap();

        builder.build().map(Self).unwrap()
    }

    pub async fn get(&self, user: AuthUser) -> Result<Client, PoolError> {
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
    user: AuthUser,
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
    user: AuthUser,
    inner: InnerTransaction<'a>,
}

impl<'a> Transaction<'a> {
    async fn set_local_role_as_user(&self) -> Result<(), TokioPgError> {
        let Self { user, inner } = self;

        let set_role_stmt = format!(r#"set local role "{user}" "#);
        inner.execute(&set_role_stmt, &[]).await?;

        Ok(())
    }

    async fn execute_as_user<T>(
        &self,
        operation: impl Future<Output = Result<T, TokioPgError>>,
    ) -> Result<T, TokioPgError> {
        self.set_local_role_as_user().await?;

        operation.await
    }

    pub async fn query_stream(
        &self,
        Sql(stmt, params): Sql<'_>,
    ) -> Result<RowStream, TokioPgError> {
        self.execute_as_user(self.inner.query_raw(&stmt, params))
            .await
    }

    pub async fn query_stream_into<T>(
        &self,
        sql: Sql<'_>,
    ) -> Result<impl Stream<Item = T>, TokioPgError>
    where
        T: FromSqlOwned,
    {
        let stream = self.query_stream(sql).await?;

        Ok(stream.map(|row| row.unwrap().get(0)))
    }

    pub async fn query(&self, Sql(stmt, params): &Sql<'_>) -> Result<Vec<Row>, TokioPgError> {
        self.execute_as_user(self.inner.query(stmt, params)).await
    }

    pub async fn query_one(&self, Sql(stmt, params): &Sql<'_>) -> Result<Row, TokioPgError> {
        self.execute_as_user(self.inner.query_one(stmt, params))
            .await
    }

    pub async fn query_one_into<T>(&self, sql: &Sql<'_>) -> Result<T, TokioPgError>
    where
        T: FromSqlOwned,
    {
        let row = self.query_one(sql).await?;

        Ok(row.get(0))
    }

    pub async fn execute(&self, Sql(stmt, params): &Sql<'_>) -> Result<u64, TokioPgError> {
        self.execute_as_user(self.inner.execute(stmt, params)).await
    }

    pub async fn execute_raw_sql(
        &self,
        stmt: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<u64, TokioPgError> {
        self.execute_as_user(self.inner.execute(stmt, params)).await
    }

    pub async fn commit(self) -> Result<(), TokioPgError> {
        self.inner.commit().await
    }

    /// Begin a nested transaction, starting a PostgreSQL savepoint
    pub async fn begin(&'a mut self) -> Result<Transaction<'a>, TokioPgError> {
        let Self { user, inner } = self;

        Ok(Self {
            user: *user,
            inner: inner.transaction().await?,
        })
    }
}
