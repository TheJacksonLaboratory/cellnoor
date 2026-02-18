use axum::{Extension, Json, extract::State};
use cellnoor_models::chromium_dataset::{ChromiumDataset, NewChromiumDataset};
use cellnoor_schema::{
    cdna, chromium_dataset_libraries, chromium_datasets, libraries, tenx_assays,
};
use diesel::{pg::Pg, prelude::*};
use diesel_async::{AsyncConnection, RunQueryDsl, scoped_futures::ScopedFutureExt};
use uuid::Uuid;

use crate::{
    api::{
        auth::AuthUser,
        routes::{
            cdna::gem_pools_to_library_specifications,
            chromium_datasets::show::select_chromium_dataset_by_id,
        },
        util::AllSame,
    },
    db::{self, DbConnection},
    state::AppState,
};

pub async fn create_chromium_dataset(
    _: State<AppState>,
    Extension(user): Extension<AuthUser>,
    mut db_conn: DbConnection,
    Json(chromium_dataset): Json<NewChromiumDataset>,
) -> Result<Json<ChromiumDataset>, db::Error> {
    let libraries_info = libraries_info(chromium_dataset.library_ids(), &db_conn).await?;

    validate_chromium_dataset(&chromium_dataset, &libraries_info)?;

    let dataset_id = db_conn
        .transaction(move |db_conn| {
            insert_chromium_dataset_and_libraries(
                libraries_info[0].project_id,
                chromium_dataset,
                db_conn,
            )
            .scope_boxed()
        })
        .await?;

    select_chromium_dataset_by_id(user.projects(), dataset_id, &db_conn)
        .await
        .map(Json)
}

fn validate_chromium_dataset(
    chromium_dataset: &NewChromiumDataset,
    libraries_info: &[LibraryInfo],
) -> Result<(), db::DataError> {
    validate_same_project(libraries_info)?;
    validate_same_gem_pool(libraries_info)?;
    validate_cmdline(chromium_dataset, libraries_info)?;
    // I absolutely hate the data source for sequencing runs, so for now, we're not
    // even gonna worry about requiring libraries to have been sequenced
    // validate_sequencing_runs_finished(chromium_dataset, libraries_info)?;

    Ok(())
}

pub async fn insert_chromium_dataset_and_libraries(
    project_id: Uuid,
    chromium_dataset: NewChromiumDataset,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<Uuid, db::Error> {
    let library_ids = chromium_dataset.library_ids().to_owned();

    let dataset_id = diesel::insert_into(chromium_datasets::table)
        .values((
            chromium_datasets::project_id.eq(project_id),
            chromium_dataset,
        ))
        .returning(chromium_datasets::id)
        .get_result(&mut db_conn)
        .await?;

    insert_chromium_dataset_libraries(dataset_id, &library_ids, db_conn).await?;

    Ok(dataset_id)
}

async fn insert_chromium_dataset_libraries(
    dataset_id: Uuid,
    library_ids: &[Uuid],
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<(), db::Error> {
    let ds_lib_map: Vec<_> = library_ids
        .iter()
        .map(|l| {
            (
                chromium_dataset_libraries::dataset_id.eq(dataset_id),
                chromium_dataset_libraries::library_id.eq(l),
            )
        })
        .collect();

    diesel::insert_into(chromium_dataset_libraries::table)
        .values(ds_lib_map)
        .execute(&mut db_conn)
        .await?;

    Ok(())
}

fn validate_same_project(libraries_info: &[LibraryInfo]) -> Result<(), db::DataError> {
    if !libraries_info.iter().map(|i| i.project_id).all_same() {
        return Err(db::DataError::new_other(
            "all libraries must be in the same project",
        ));
    }

    Ok(())
}

fn validate_same_gem_pool(libraries_info: &[LibraryInfo]) -> Result<(), db::DataError> {
    if !libraries_info.iter().map(|i| i.cdna.gem_pool_id).all_same() {
        return Err(db::DataError::new_other(
            "all libraries must come from the same GEMs pool",
        ));
    }

    Ok(())
}

fn validate_cmdline(
    chromium_dataset: &NewChromiumDataset,
    libraries_info: &[LibraryInfo],
) -> Result<(), db::DataError> {
    let mut expected_cmdlines = libraries_info
        .iter()
        .flat_map(|i| i.assay.cmdlines.iter().flatten().flatten())
        .map(String::as_str);

    let found_cmdline: &str = chromium_dataset.cmdline().into();

    // We could collect the `expected_cmdlines` into a set but it's kind of
    // pointless because that requires an iteration anyways
    if !expected_cmdlines.any(|expected| expected == found_cmdline) {
        return Err(db::DataError::new_other("invalid cmdline used"));
    }

    Ok(())
}

// #[allow(dead_code)]
// fn validate_sequencing_runs_finished(
//     chromium_dataset: &NewChromiumDataset,
//     libraries_info: &[LibraryInfo],
// ) -> Result<(), db::DataError> {
//     if libraries_info.is_empty() {
//         return Err(db::DataError::new_other("libraries not sequenced"));
//     }

//     for lib_info in libraries_info {
//         let Some(finished_at) = lib_info.sequencing_run.finished_at else {
//             return Err(db::DataError::new_other(&format!(
//                 "library {} was not sequenced",
//                 lib_info.id
//             )));
//         };

//         validate_timestamps(
//             (finished_at, "sequencing_run_finished_at"),
//             (chromium_dataset.delivered_at(), "dataset_delivered_at"),
//         )?;
//     }

//     Ok(())
// }

// #[derive(Selectable, Queryable)]
// #[diesel(check_for_backend(Pg), table_name = sequencing_runs)]
// struct SequencingRunInfo {
//     #[diesel(deserialize_as = jiff_diesel::NullableTimestamp)]
//     finished_at: Option<Timestamp>,
// }

#[derive(Selectable, Queryable)]
#[diesel(check_for_backend(Pg), table_name = cdna)]
struct CdnaInfo {
    gem_pool_id: Option<Uuid>,
}

#[derive(Selectable, Queryable)]
#[diesel(check_for_backend(Pg), table_name = tenx_assays)]
struct AssayInfo {
    cmdlines: Option<Vec<Option<String>>>,
}

#[derive(HasQuery)]
#[diesel(check_for_backend(Pg), table_name = libraries, base_query = libraries_to_library_specifications())]
struct LibraryInfo {
    #[diesel(embed)]
    cdna: CdnaInfo,
    // #[diesel(embed)]
    // sequencing_run: SequencingRunInfo,
    #[diesel(embed)]
    assay: AssayInfo,
    project_id: Uuid,
}

async fn libraries_info(
    library_ids: &[Uuid],
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<Vec<LibraryInfo>, db::Error> {
    Ok(LibraryInfo::query()
        .filter(libraries::id.eq_any(library_ids))
        .distinct()
        .load(&mut db_conn)
        .await?)
}

#[diesel::dsl::auto_type]
fn libraries_to_library_specifications() -> _ {
    libraries::table.inner_join(cdna::table.inner_join(gem_pools_to_library_specifications()))
}
