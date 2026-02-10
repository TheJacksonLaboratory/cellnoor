use axum::{Extension, Json, extract::State};
use cellnoor_models::library::{Library, NewLibrary};
use cellnoor_schema::{cdna, library_preparers};
use diesel::{pg::Pg, prelude::*};
use diesel_async::{AsyncConnection, RunQueryDsl, scoped_futures::ScopedFutureExt};
use jiff::Timestamp;
use uuid::Uuid;

use crate::{
    api::{
        auth::AuthUser,
        routes::{
            cdna::{NucleicAcidParentInfo, gem_pools_to_library_specs, validate_volume},
            libraries::show::select_library_by_id,
        },
        util::validate_timestamps,
    },
    db::{self, DbConnection},
    initial_data::index_sets::IndexSetName,
    state::AppState,
};

pub(super) async fn create_library(
    _: State<AppState>,
    mut db_conn: DbConnection,
    Extension(user): Extension<AuthUser>,
    Json(library): Json<NewLibrary>,
) -> Result<Json<Library>, db::Error> {
    let cdna_info = cdna_info(library.cdna_id(), &db_conn).await?;

    validate_volume(
        &cdna_info.parent_info,
        library.volume_µl(),
        cdna_info
            .parent_info
            .library_type_specification
            .library_volume_µl(),
    )?;

    validate_index_kit(
        cdna_info.parent_info.library_type_specification.index_kit(),
        (
            library.dual_index_set_name(),
            library.single_index_set_name(),
        ),
    )?;

    validate_timestamps(
        (library.prepared_at(), "library_prepared_at"),
        (cdna_info.prepared_at, "chromium_run_at"),
    )?;

    let library_id = db_conn
        .transaction(|db_conn| {
            insert_library_and_preparers(cdna_info.project_id, library, db_conn).scope_boxed()
        })
        .await?;

    select_library_by_id(user.projects(), library_id, &db_conn)
        .await
        .map(Json)
}

pub async fn insert_library_and_preparers(
    project_id: Uuid,
    library: NewLibrary,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<Uuid, db::Error> {
    use cellnoor_schema::libraries;

    let preparer_ids = library.preparer_ids().to_vec();

    let library_id = diesel::insert_into(libraries::table)
        .values((libraries::project_id.eq(project_id), library))
        .returning(libraries::id)
        .get_result(&mut db_conn)
        .await?;

    insert_library_preparers(library_id, &preparer_ids, db_conn).await?;

    Ok(library_id)
}

async fn insert_library_preparers(
    library_id: Uuid,
    preparer_ids: &[Uuid],
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<(), db::Error> {
    let preparer_mappings: Vec<_> = preparer_ids
        .iter()
        .map(|p| {
            (
                library_preparers::library_id.eq(library_id),
                library_preparers::prepared_by.eq(p),
            )
        })
        .collect();

    diesel::insert_into(library_preparers::table)
        .values(preparer_mappings)
        .execute(&mut db_conn)
        .await?;

    Ok(())
}

fn validate_index_kit(
    expected_index_kit: &str,
    (dual_index_set, single_index_set): (Option<&str>, Option<&str>),
) -> Result<(), db::DataError> {
    if dual_index_set.is_some_and(|_| single_index_set.is_some()) {
        return Err(db::DataError::new_other(
            "library cannot have two index sets",
        ));
    }

    let index_set = dual_index_set
        .or(single_index_set)
        .ok_or(db::DataError::new_other("library must have index set"))?;

    let found_index_kit = index_set.kit_name().map_err(|_| {
        db::DataError::new_other(&format!(
            "expected index kit {expected_index_kit}, found none"
        ))
    })?;

    if expected_index_kit != found_index_kit {
        return Err(db::DataError::new_other(&format!(
            "expected index kit {expected_index_kit}, found {found_index_kit}"
        )));
    }

    Ok(())
}

#[derive(HasQuery)]
#[diesel(check_for_backend(Pg), table_name = cdna, base_query = cdna::table.inner_join(gem_pools_to_library_specs()))]
struct CdnaInfo {
    #[diesel(embed)]
    parent_info: NucleicAcidParentInfo,
    #[diesel(deserialize_as = jiff_diesel::Timestamp)]
    prepared_at: Timestamp,
    project_id: Uuid,
}

async fn cdna_info(
    cdna_id: Uuid,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<CdnaInfo, db::Error> {
    Ok(CdnaInfo::query()
        .filter(cdna::id.eq(cdna_id))
        .first(&mut db_conn)
        .await?)
}
