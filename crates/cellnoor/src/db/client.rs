use deadpool_postgres::{
    GenericClient, Object as InnerClient, Pool as InnerPool, PoolError,
    Transaction as InnerTransaction,
    tokio_postgres::{Error as TokioPgError, Row, RowStream, types::ToSql},
};
use futures::{Stream, StreamExt};
use postgres_types::FromSqlOwned;

use crate::{auth::AuthUser, db::Sql};

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

    /// A connection that acts on behalf of nobody.
    ///
    /// Row-level security denies it everything, so it's only good for calling
    /// `security definer` functions. Authenticating an API key needs this
    /// because the user isn't known until the key has been found.
    pub(crate) async fn unauthenticated(&self) -> Result<InnerClient, PoolError> {
        self.0.get().await
    }
}

/// A database client that acts on behalf of a user.
#[derive(Debug)]
pub struct Client {
    user: AuthUser,
    inner: InnerClient,
}

impl Client {
    /// Begin a transaction that runs as the authenticated user.
    ///
    /// Every row-level security policy reads `app.user_id`, which is only set
    /// for the duration of the transaction. Nothing is saved unless
    /// [Transaction::commit] is called.
    pub async fn begin(&'_ mut self) -> Result<Transaction<'_>, TokioPgError> {
        let Self { user, inner } = self;

        let inner = inner.transaction().await?;
        inner
            .execute(
                "select set_config('app.user_id', $1, true)",
                &[&user.id().to_string()],
            )
            .await?;

        Ok(Transaction { user: *user, inner })
    }
}

pub struct Transaction<'a> {
    user: AuthUser,
    inner: InnerTransaction<'a>,
}

impl<'a> Transaction<'a> {
    pub async fn query_stream(
        &self,
        Sql(stmt, params): Sql<'_>,
    ) -> Result<RowStream, TokioPgError> {
        self.inner.query_raw(&stmt, params).await
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
        self.inner.query(stmt, params).await
    }

    pub async fn query_one(&self, Sql(stmt, params): &Sql<'_>) -> Result<Row, TokioPgError> {
        self.inner.query_one(stmt, params).await
    }

    pub async fn query_one_into<T>(&self, sql: &Sql<'_>) -> Result<T, TokioPgError>
    where
        T: FromSqlOwned,
    {
        let row = self.query_one(sql).await?;

        Ok(row.get(0))
    }

    pub async fn execute(&self, Sql(stmt, params): &Sql<'_>) -> Result<u64, TokioPgError> {
        self.inner.execute(stmt, params).await
    }

    pub async fn execute_raw_sql(
        &self,
        stmt: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<u64, TokioPgError> {
        self.inner.execute(stmt, params).await
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

    pub fn user(&self) -> AuthUser {
        self.user
    }
}
