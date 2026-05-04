use axum::{Extension, Json, extract::State};
use cellnoor_models::{
    cdna::{Cdna, NewCdna},
    tenx_assay::{LibraryType, LibraryTypeSpecification},
};
use cellnoor_schema::{
    cdna, cdna_preparers, chromium_runs, gem_pools, library_type_specifications as lib_specs,
    tenx_assays,
};
use diesel::{pg::Pg, prelude::*};
use diesel_async::{AsyncConnection, RunQueryDsl, scoped_futures::ScopedFutureExt};
use jiff::Timestamp;
use uuid::Uuid;

use crate::{
    api::{auth::AuthUser, routes::cdna::show::select_cdna_by_id, util::validate_timestamps},
    db::{self, DbConnection},
    state::AppState,
};

pub async fn create_cdna(
    _: State<AppState>,
    mut db_conn: DbConnection,
    Extension(user): Extension<AuthUser>,
    Json(new_cdna): Json<NewCdna>,
) -> Result<Json<Cdna>, db::Error> {
    let Some(gem_pool_id) = new_cdna.gem_pool_id() else {
        return Err(db::DataError::new_other(
            "cDNA without GEMs pool not supported",
        ))?;
    };

    let parent_info =
        nucleic_acid_parent_info(gem_pool_id, new_cdna.library_type(), &db_conn).await?;

    validate_volume(
        &parent_info,
        new_cdna.volume_µl(),
        parent_info.library_type_specification.cdna_volume_µl(),
    )?;
    validate_timestamps(
        (parent_info.chromium_run.run_at, "chromium_run_at"),
        (new_cdna.prepared_at(), "cdna_prepared_at"),
    )?;

    let cdna_id = db_conn
        .transaction(|db_conn| {
            insert_cdna_and_preparers(parent_info.chromium_run.project_id, new_cdna, db_conn)
                .scope_boxed()
        })
        .await?;

    select_cdna_by_id(user.projects(), cdna_id, &db_conn)
        .await
        .map(Json)
}

pub async fn insert_cdna_and_preparers(
    project_id: Uuid,
    new_cdna: NewCdna,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<Uuid, db::Error> {
    let preparer_ids = new_cdna.preparer_ids().to_vec();

    let cdna_id = diesel::insert_into(cdna::table)
        .values((cdna::project_id.eq(project_id), new_cdna))
        .returning(cdna::id)
        .get_result(&mut db_conn)
        .await?;

    insert_cdna_preparers(cdna_id, &preparer_ids, db_conn).await?;

    Ok(cdna_id)
}

async fn insert_cdna_preparers(
    cdna_id: Uuid,
    preparer_ids: &[Uuid],
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<(), db::Error> {
    let preparer_mappings: Vec<_> = preparer_ids
        .iter()
        .map(|p| {
            (
                cdna_preparers::cdna_id.eq(cdna_id),
                cdna_preparers::prepared_by.eq(p),
            )
        })
        .collect();

    diesel::insert_into(cdna_preparers::table)
        .values(preparer_mappings)
        .execute(&mut db_conn)
        .await?;

    Ok(())
}

#[derive(HasQuery)]
#[diesel(table_name = chromium_runs, check_for_backend(Pg), base_query=chromium_runs::table.inner_join(tenx_assays::table))]
pub struct ChromiumRunInfo {
    #[diesel(deserialize_as = jiff_diesel::Timestamp)]
    pub run_at: Timestamp,
    pub project_id: Uuid,
}

#[must_use]
#[diesel::dsl::auto_type]
pub fn gem_pools_to_library_specifications() -> _ {
    gem_pools::table.inner_join(
        chromium_runs::table.inner_join(tenx_assays::table.inner_join(lib_specs::table)),
    )
}

#[derive(HasQuery)]
#[diesel(table_name = gem_pools, check_for_backend(Pg), base_query=gem_pools_to_library_specifications())]
pub struct NucleicAcidParentInfo {
    #[diesel(embed)]
    pub library_type_specification: LibraryTypeSpecification,
    #[diesel(embed)]
    pub chromium_run: ChromiumRunInfo,
}

async fn nucleic_acid_parent_info(
    gem_pool_id: Uuid,
    library_type: LibraryType,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<NucleicAcidParentInfo, db::Error> {
    Ok(NucleicAcidParentInfo::query()
        .filter(gem_pools::id.eq(gem_pool_id))
        .filter(lib_specs::library_type.eq(library_type))
        .first(&mut db_conn)
        .await?)
}

pub fn validate_volume(
    NucleicAcidParentInfo {
        library_type_specification,
        ..
    }: &NucleicAcidParentInfo,
    volume: u8,
    expected_volume: u16,
) -> Result<(), db::DataError> {
    if u16::from(volume) != expected_volume {
        let library_type: &str = library_type_specification.library_type().into();
        return Err(db::DataError::new_other(&format!(
            "for library type {library_type}, expected cDNA volume of {expected_volume}"
        )));
    }

    Ok(())
}
