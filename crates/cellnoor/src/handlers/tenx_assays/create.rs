use axum::{Json, extract::State};
use cellnoor_types::tenx_assay::creation::{LibraryTypeSpecification, NewTenxAssay};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, AsFieldValuePairs, FieldValuePairs, insert_into_no_returning},
    error::{Error, ErrorInner},
    handlers::tenx_assays::create::chromium::insert_chromium_assay,
    state::AppState,
};

#[cfg(test)]
pub use chromium::tests::insert_test_chromium_assay;

mod chromium;

pub async fn create_tenx_assay(
    State(state): State<AppState>,
    user: AuthUser,
    Json(new): Json<NewTenxAssay>,
) -> Result<Json<Uuid>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let id = insert_tenx_assay(&tx, new).await?;

    tx.commit().await?;

    Ok(Json(id))
}

pub async fn insert_tenx_assay(
    tx: &db::Transaction<'_>,
    new: NewTenxAssay,
) -> Result<Uuid, ErrorInner> {
    match new {
        NewTenxAssay::Chromium(chromium) => insert_chromium_assay(tx, &chromium).await,
    }
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
