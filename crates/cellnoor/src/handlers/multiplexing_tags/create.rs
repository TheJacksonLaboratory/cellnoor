use axum::{Json, extract::State};
use cellnoor_types::multiplexing_tag::{MultiplexingTag, NewMultiplexingTag};
use postgres_types::ToSql;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, AsFieldValuePairs, FieldValuePairs, SqlBuilder, insert_into},
    error::{Error, ErrorInner},
    state::AppState,
};

pub async fn create_multiplexing_tag(
    State(state): State<AppState>,
    user: AuthUser,
    Json(new): Json<NewMultiplexingTag>,
) -> Result<Json<MultiplexingTag>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = insert_multiplexing_tag(&tx, &new).await?;

    tx.commit().await?;

    Ok(Json(response))
}

async fn insert_multiplexing_tag(
    tx: &db::Transaction<'_>,
    NewMultiplexingTag { tag_id, type_ }: &NewMultiplexingTag,
) -> Result<MultiplexingTag, ErrorInner> {
    static INSERT_MULTIPLEXING_TAG: SqlBuilder = SqlBuilder::new(
        "insert into multiplexing_tag (tag_id, type) values ($1, $2) returning multiplexing_tag",
    );

    let params: Vec<&(dyn ToSql + Sync)> = vec![tag_id, type_];

    Ok(tx
        .query_one_into(&INSERT_MULTIPLEXING_TAG.finish_with_params(params))
        .await?)
}

impl AsFieldValuePairs<&'static str, 2> for NewMultiplexingTag {
    fn as_field_value_pairs(&self) -> FieldValuePairs<'_, &'static str, 2> {
        let Self { tag_id, type_ } = self;

        [("tag_id", tag_id), ("type", type_)]
    }
}

#[cfg(test)]
pub mod tests {
    use cellnoor_types::{
        multiplexing_tag::{MultiplexingTag, NewMultiplexingTag},
        suspension_pool::MultiplexingTagType,
    };
    use uuid::Uuid;

    use crate::{
        db,
        error::ErrorInner,
        handlers::multiplexing_tags::create::insert_multiplexing_tag,
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    pub async fn insert_test_multiplexing_tag(
        tx: &db::Transaction<'_>,
    ) -> Result<MultiplexingTag, ErrorInner> {
        let new = NewMultiplexingTag {
            tag_id: Uuid::new_v4().to_string().to_nonempty_string(),
            type_: MultiplexingTagType::FlexBarcode,
        };

        insert_multiplexing_tag(tx, &new).await
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        insert_test_multiplexing_tag(&tx).await.unwrap();
    }
}
