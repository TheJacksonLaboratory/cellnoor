use axum::{Json, extract::State};
use cellnoor_types::tenx_assay::{
    TenxAssay,
    creation::{LibraryTypeSpecification, NewTenxAssay},
};
#[cfg(test)]
pub use chromium::tests::insert_test_chromium_assay;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, AsFieldValuePairs, BaseSqlStmt, FieldValuePairs, insert_into_no_returning},
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

pub async fn insert_tenx_assay(
    tx: &db::Transaction<'_>,
    new: &NewTenxAssay,
) -> Result<TenxAssay, ErrorInner> {
    let assay_id = match new {
        NewTenxAssay::Chromium(chromium) => insert_chromium_assay(tx, chromium).await?,
    };

    let assay = tx
        .query_one_into(
            &BaseSqlStmt::new("select tenx_assay from tenx_assay where id = $1")
                .finish_with_params(vec![&assay_id]),
        )
        .await?;

    Ok(assay)
}

async fn insert_library_type_specification(
    tx: &db::Transaction<'_>,
    record: &NewLibraryTypeSpecificationRecord<'_>,
) -> Result<(), ErrorInner> {
    insert_into_no_returning(tx, "library_type_specification", record).await?;

    Ok(())
}

struct NewLibraryTypeSpecificationRecord<'a> {
    assay_id: Uuid,
    spec: &'a LibraryTypeSpecification,
}

impl<'a> AsFieldValuePairs<&'static str, 5> for NewLibraryTypeSpecificationRecord<'a> {
    fn as_field_value_pairs(&self) -> FieldValuePairs<'_, &'static str, 5> {
        let Self { assay_id, spec } = self;

        [
            ("assay_id", assay_id),
            ("library_type", &spec.library_type),
            ("index_kit", &spec.index_kit),
            ("cdna_volume_µl", &spec.cdna_volume_µl),
            ("library_volume_µl", &spec.library_volume_µl),
        ]
    }
}
