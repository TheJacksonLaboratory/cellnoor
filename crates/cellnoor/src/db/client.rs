use cellnoor_types::{
    Relation,
    filter::AsPredicate,
    query::{ComplexQuery, OrderField},
};
use deadpool_postgres::{
    GenericClient, Object as InnerClient, Pool as InnerPool, PoolError,
    Transaction as InnerTransaction,
    tokio_postgres::{Error as TokioPgError, Row},
};
use postgres_types::FromSqlOwned;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{FilterableSqlBuilder, Insert, Sql, insert, update},
    error::ErrorInner,
};

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

impl Transaction<'_> {
    /// Run a filterable select, decoding each row's single column.
    ///
    /// Every such statement selects a relation's whole row as a composite, so
    /// `T` is the type that maps to that relation.
    pub async fn select<T, P, O>(
        &self,
        sql: &FilterableSqlBuilder,
        query: &ComplexQuery<P, O>,
    ) -> Result<Vec<T>, ErrorInner>
    where
        T: FromSqlOwned,
        P: AsPredicate,
        O: OrderField,
    {
        self.query_into(&sql.finish_with_query(query)).await
    }

    /// Run a statement that takes no filter, decoding each row's single column.
    pub async fn query_into<T>(&self, sql: &Sql<'_>) -> Result<Vec<T>, ErrorInner>
    where
        T: FromSqlOwned,
    {
        let rows = self.query(sql).await?;

        Ok(rows.iter().map(|row| row.get(0)).collect())
    }

    /// Run a filterable select, for the few statements whose columns are read
    /// by name rather than decoded as one composite.
    pub async fn select_rows<P, O>(
        &self,
        sql: &FilterableSqlBuilder,
        query: &ComplexQuery<P, O>,
    ) -> Result<Vec<Row>, ErrorInner>
    where
        P: AsPredicate,
        O: OrderField,
    {
        self.query(&sql.finish_with_query(query)).await
    }

    /// Select the one row matching a predicate, shaped by the same function
    /// that the index endpoint uses.
    pub async fn select_one<T, P, O>(
        &self,
        predicate: P,
        select: impl AsyncFn(&Self, &ComplexQuery<P, O>) -> Result<Vec<T>, ErrorInner>,
    ) -> Result<T, ErrorInner>
    where
        O: OrderField,
    {
        let mut records = select(self, &ComplexQuery::from_filter(predicate)).await?;

        if records.len() != 1 {
            return Err(ErrorInner::ResourceNotFound);
        }

        Ok(records.swap_remove(0))
    }

    pub async fn query_one(&self, Sql(stmt, params): &Sql<'_>) -> Result<Row, ErrorInner> {
        Ok(self.inner.query_one(stmt, params).await?)
    }

    pub async fn query_one_into<T>(&self, sql: &Sql<'_>) -> Result<T, ErrorInner>
    where
        T: FromSqlOwned,
    {
        Ok(self.query_one(sql).await?.get(0))
    }

    async fn query(&self, Sql(stmt, params): &Sql<'_>) -> Result<Vec<Row>, ErrorInner> {
        Ok(self.inner.query(stmt, params).await?)
    }
}

impl Transaction<'_> {
    /// Insert one row and return the id the database assigned it.
    pub async fn insert_returning_id<T>(&self, record: &T) -> Result<Uuid, ErrorInner>
    where
        T: Insert,
    {
        let fields = record.fields();

        self.query_one_into(&insert::insert_stmt(T::NAME, &fields, Some("id")))
            .await
    }

    /// Insert one row into a relation that has no id to return.
    pub async fn insert<T>(&self, record: &T) -> Result<(), ErrorInner>
    where
        T: Insert,
    {
        let fields = record.fields();

        self.execute(&insert::insert_stmt(T::NAME, &fields, None))
            .await?;

        Ok(())
    }

    /// Insert every row in one statement.
    pub async fn insert_many<T>(&self, records: &[T]) -> Result<(), ErrorInner>
    where
        T: Insert,
    {
        self.insert_rows(records, false).await
    }

    /// Insert every row in one statement, skipping the rows that are already
    /// there. Only for relations whose primary key is the whole row, where
    /// re-inserting a row means nothing.
    pub async fn insert_many_on_conflict_do_nothing<T>(
        &self,
        records: &[T],
    ) -> Result<(), ErrorInner>
    where
        T: Insert,
    {
        self.insert_rows(records, true).await
    }

    pub async fn update<T>(&self, id: Uuid, record: &T) -> Result<(), ErrorInner>
    where
        T: Insert,
    {
        let fields = record.fields();

        if fields.is_empty() {
            return Err(ErrorInner::Other {
                message: format!("no update provided for {}", T::NAME),
                sql_state: None,
            });
        }

        let n = self
            .execute(&update::update_stmt(T::NAME, &id, &fields))
            .await?;

        if n == 0 {
            return Err(ErrorInner::ResourceNotFound);
        }

        Ok(())
    }

    pub async fn delete<T>(&self, id: Uuid) -> Result<(), ErrorInner>
    where
        T: Relation,
    {
        let stmt = format!("delete from {} where id = $1", T::NAME);

        let n = self.execute(&Sql(stmt, vec![&id])).await?;

        if n == 0 {
            return Err(ErrorInner::ResourceNotFound);
        }

        Ok(())
    }

    pub async fn execute(&self, Sql(stmt, params): &Sql<'_>) -> Result<u64, ErrorInner> {
        Ok(self.inner.execute(stmt, params).await?)
    }

    async fn insert_rows<T>(
        &self,
        records: &[T],
        on_conflict_do_nothing: bool,
    ) -> Result<(), ErrorInner>
    where
        T: Insert,
    {
        if records.is_empty() {
            return Ok(());
        }

        let rows: Vec<_> = records.iter().map(Insert::fields).collect();

        self.execute(&insert::insert_many_stmt(
            T::NAME,
            &rows,
            on_conflict_do_nothing,
        ))
        .await?;

        Ok(())
    }
}

impl<'a> Transaction<'a> {
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
