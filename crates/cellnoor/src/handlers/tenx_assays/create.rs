use axum::{Json, extract::State};
use cellnoor_types::{
    Relation,
    tenx_assay::{
        TenxAssay,
        creation::{LibraryTypeSpecification, NewTenxAssay},
    },
};
#[cfg(test)]
pub use chromium::tests::insert_test_chromium_assay;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, FieldValues, Insert, SqlBuilder},
    error::{Error, ErrorInner},
    handlers::tenx_assays::create::chromium::insert_chromium_assay,
    state::AppState,
};

mod chromium;

pub async fn create_tenx_assay(
    State(state): State<AppState>,
    user: AuthUser,
    Json(new): Json<NewTenxAssay>,
) -> Result<Json<TenxAssay>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = insert_tenx_assay(&tx, &new).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

async fn insert_tenx_assay(
    tx: &db::Transaction<'_>,
    new: &NewTenxAssay,
) -> Result<TenxAssay, ErrorInner> {
    let assay_id = match new {
        NewTenxAssay::Chromium(chromium) => insert_chromium_assay(tx, chromium).await?,
    };

    let assay = tx
        .query_one_into(
            &SqlBuilder::new("select tenx_assay from tenx_assay where id = $1")
                .finish_with_params(vec![&assay_id]),
        )
        .await?;

    Ok(assay)
}

async fn insert_library_type_specification(
    tx: &db::Transaction<'_>,
    record: &NewLibraryTypeSpecificationRecord<'_>,
) -> Result<(), ErrorInner> {
    tx.insert(record).await?;

    Ok(())
}

struct NewLibraryTypeSpecificationRecord<'a> {
    assay_id: Uuid,
    spec: &'a LibraryTypeSpecification,
}

impl<'a> Relation for NewLibraryTypeSpecificationRecord<'a> {
    const NAME: &'static str = "library_type_specification";
}

impl<'a> Insert for NewLibraryTypeSpecificationRecord<'a> {
    type Field = &'static str;

    fn fields(&self) -> FieldValues<'_, Self::Field> {
        let Self { assay_id, spec } = self;

        vec![
            ("assay_id", assay_id),
            ("library_type", &spec.library_type),
            ("index_kit", &spec.index_kit),
            ("cdna_volume_µl", &spec.cdna_volume_µl),
            ("library_volume_µl", &spec.library_volume_µl),
        ]
    }
}
