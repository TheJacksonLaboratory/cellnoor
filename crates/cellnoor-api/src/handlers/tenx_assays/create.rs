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
    db::{self, DbError, FieldValues, Insert, Sql},
    handlers::tenx_assays::create::chromium::insert_chromium_assay,
    state::AppState,
};

mod chromium;

pub async fn create_tenx_assay(
    State(state): State<AppState>,
    user: AuthUser,
    crate::extract::JsonExtractor(new): crate::extract::JsonExtractor<NewTenxAssay>,
) -> Result<Json<TenxAssay>, DbError> {
    state
        .in_transaction(user, async |tx| insert_tenx_assay(tx, &new).await)
        .await
}

async fn insert_tenx_assay(
    tx: &db::Transaction<'_>,
    new: &NewTenxAssay,
) -> Result<TenxAssay, DbError> {
    let assay_id = match new {
        NewTenxAssay::Chromium(chromium) => insert_chromium_assay(tx, chromium).await?,
    };

    let assay = tx
        .query_one_into(&Sql::new(
            "select tenx_assay from tenx_assay where id = $1",
            vec![&assay_id],
        ))
        .await?;

    Ok(assay)
}

async fn insert_library_type_specification(
    tx: &db::Transaction<'_>,
    record: &NewLibraryTypeSpecificationRecord<'_>,
) -> Result<(), DbError> {
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
