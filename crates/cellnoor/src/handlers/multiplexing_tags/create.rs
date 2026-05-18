use axum::{Json, extract::State};
use cellnoor_types::multiplexing_tag::NewMultiplexingTag;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, AsFieldValuePairs, FieldValuePairs, insert_into},
    error::{Error, ErrorInner},
    state::AppState,
};

pub async fn create_multiplexing_tag(
    State(state): State<AppState>,
    user: AuthUser,
    Json(new): Json<NewMultiplexingTag>,
) -> Result<Json<Uuid>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let id = insert_multiplexing_tag(&tx, &new).await?;

    tx.commit().await?;

    Ok(Json(id))
}

pub async fn insert_multiplexing_tag(
    tx: &db::Transaction<'_>,
    new: &NewMultiplexingTag,
) -> Result<Uuid, ErrorInner> {
    Ok(insert_into(tx, "multiplexing_tag", new).await?)
}

impl AsFieldValuePairs<&'static str, 2> for NewMultiplexingTag {
    fn as_field_value_pairs(&self) -> FieldValuePairs<'_, &'static str, 2> {
        let Self {
            id: _,
            tag_id,
            type_,
        } = self;

        [("tag_id", tag_id), ("type", type_)]
    }
}

#[cfg(test)]
pub mod tests {
    use cellnoor_types::{
        id::NoId,
        multiplexing_tag::{MultiplexingTagType, NewMultiplexingTag},
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
    ) -> Result<Uuid, ErrorInner> {
        let new = NewMultiplexingTag {
            id: NoId {},
            tag_id: Uuid::new_v4().to_string().to_nonempty_string(),
            type_: MultiplexingTagType::FlexBarcode,
        };

        insert_multiplexing_tag(tx, &new).await
    }

    #[tokio::test]
    async fn insert() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        insert_test_multiplexing_tag(&tx).await.unwrap();
    }
}
