use axum::{Json, extract::State};
use cellnoor_types::chromium_run::{
    ChromiumRunDetailed, ChromiumRunQuery, GemWell, SavedChromiumRunRecord,
    SavedGemWellWithSpecimensRecord,
};
use deadpool_postgres::tokio_postgres::Row;
use futures::StreamExt;

use crate::{
    auth::AuthUser,
    db::{self, FilterableSqlBuilder},
    error::{Error, ErrorInner},
    handlers::{
        chromium_runs::index_compact::chromium_run_links,
        suspension_pools::index_compact::tagged_specimen_from_record,
    },
    state::AppState,
};

pub async fn index_chromium_runs_detailed(
    State(state): State<AppState>,
    user: AuthUser,
    Json(mut query): Json<ChromiumRunQuery>,
) -> Result<Json<Vec<ChromiumRunDetailed>>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_chromium_runs_detailed(&tx, &mut query)
        .await
        .map(Json)?;

    tx.commit().await?;

    Ok(response)
}

// Visibility required for tests
pub(in super::super) async fn select_chromium_runs_detailed(
    tx: &db::Transaction<'_>,
    query: &mut ChromiumRunQuery,
) -> Result<Vec<ChromiumRunDetailed>, ErrorInner> {
    static SELECT_DETAILED_CHROMIUM_RUNS: FilterableSqlBuilder =
        FilterableSqlBuilder::new(include_str!("index/select_detailed.sql"));

    let sql = SELECT_DETAILED_CHROMIUM_RUNS.finish_with_query(query);

    let stream = tx.query_stream(sql).await?;
    Ok(stream
        .map(|row| row.map(map_detailed_row).unwrap())
        .collect()
        .await)
}

fn map_detailed_row(row: Row) -> ChromiumRunDetailed {
    let record: SavedChromiumRunRecord = row.get("chromium_run");
    let assay = row.get("tenx_assay");
    let gem_wells: Vec<SavedGemWellWithSpecimensRecord> = row.get("gem_wells");

    ChromiumRunDetailed {
        links: chromium_run_links(record.id),
        record,
        assay,
        gem_wells: gem_wells
            .into_iter()
            .map(|g| GemWell {
                record: g.gem_well,
                specimens: g
                    .specimens
                    .into_iter()
                    .map(tagged_specimen_from_record)
                    .collect(),
            })
            .collect(),
    }
}

#[cfg(test)]
mod test {
    use std::{collections::HashSet, hash::RandomState};

    use cellnoor_types::chromium_run::creation::{
        NewChromiumRun,
        ocm::{NewOcmChipLoading, NewOcmGemWell, OcmBarcodeId},
    };
    use nonempty::NonemptyBoundedVec;
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use crate::{
        handlers::{
            chromium_runs::{
                create::test::{
                    insert_test_mixed_chromium_run, insert_test_ocm_chromium_run,
                    insert_test_standard_chromium_run, loading_common, new_common,
                },
                show::select_chromium_run_by_id,
            },
            suspensions::create::test::insert_test_suspension_and_specimen,
            tenx_assays::create::insert_test_chromium_assay,
        },
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn select_detailed_standard() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, run) = insert_test_standard_chromium_run(&tx, |_| ())
            .await
            .unwrap();

        let detailed = select_chromium_run_by_id(&tx, *run.record.id)
            .await
            .unwrap();
        let gem_wells = detailed.gem_wells;

        assert_eq!(gem_wells.len(), 2);

        // Ensure that the returned specimens are all different and that they have
        // different multiplexing tags, so aggregate all the specimens
        let mut specimens = gem_wells[0].specimens.clone();
        specimens.extend_from_slice(&gem_wells[1].specimens);

        assert_eq!(specimens.len(), 3);

        let set: HashSet<_, RandomState> =
            HashSet::from_iter(specimens.iter().map(|s| s.specimen.record.id));
        assert_eq!(set.len(), 3);

        let set: HashSet<_, RandomState> = HashSet::from_iter(
            specimens
                .iter()
                .filter_map(|s| s.multiplexing_tag.as_ref())
                .map(|t| t.id),
        );
        assert_eq!(set.len(), 2);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn select_detailed_ocm() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, run) = insert_test_ocm_chromium_run(&tx, |_| ()).await.unwrap();
        let detailed = select_chromium_run_by_id(&tx, *run.record.id)
            .await
            .unwrap();
        let gem_wells = detailed.gem_wells;

        assert_eq!(gem_wells.len(), 2);

        // Ensure that the two returned specimens are different and that they have
        // different OCM barcode IDs
        let specimens = &gem_wells[0].specimens;
        assert_eq!(specimens.len(), 2);
        assert_ne!(
            specimens[0].specimen.record.id,
            specimens[1].specimen.record.id
        );
        assert_ne!(
            specimens[0].ocm_barcode_id.unwrap(),
            specimens[1].ocm_barcode_id.unwrap()
        )
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn select_detailed_mixed() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, run) = insert_test_mixed_chromium_run(&tx, |_| ()).await.unwrap();
        let detailed = select_chromium_run_by_id(&tx, *run.record.id)
            .await
            .unwrap();
        let gem_wells = detailed.gem_wells;

        assert_eq!(gem_wells.len(), 2);
        assert_eq!(gem_wells[0].specimens.len(), 1);
        assert_eq!(gem_wells[1].specimens.len(), 1);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn ocm_same_suspension() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, run) = insert_test_ocm_chromium_run(&tx, point_second_loading_at_first_suspension)
            .await
            .unwrap();

        let detailed = select_chromium_run_by_id(&tx, *run.record.id)
            .await
            .unwrap();
        let gem_wells = detailed.gem_wells;

        assert_eq!(gem_wells.len(), 2);

        let specimens = &gem_wells[0].specimens;
        assert_eq!(specimens.len(), 2);

        assert_eq!(
            *specimens[0].specimen.record.id,
            *specimens[1].specimen.record.id
        );

        assert_ne!(specimens[0].ocm_barcode_id, specimens[1].ocm_barcode_id);

        // The second GEM well is loaded with two different specimens, but the first
        // specimen is the same as the two specimens loaded in the first GEM well
        assert_eq!(gem_wells[1].specimens[0], specimens[0]);
    }

    fn point_second_loading_at_first_suspension(new: &mut NewChromiumRun) {
        // We do this for both the GEM wells in the Chromium run
        fn get_suspension_id(
            chromium_run: &mut NewChromiumRun,
            gem_well_idx: usize,
            loading_idx: usize,
        ) -> &mut Uuid {
            let NewChromiumRun::OnChipMultiplexing { gem_wells, .. } = chromium_run else {
                unreachable!("expected NewChromiumRun::OnChipMultiplexing");
            };

            let gem_well = &mut gem_wells.as_mut()[gem_well_idx];
            let loading = gem_well.loading.as_mut();

            let NewOcmChipLoading::Suspension { suspension_id, .. } = &mut loading[loading_idx]
            else {
                unreachable!()
            };

            suspension_id
        }

        let first_suspension_id = get_suspension_id(new, 0, 0).to_owned();
        let second_suspension_id = get_suspension_id(new, 0, 1);
        *second_suspension_id = first_suspension_id;

        let first_suspension_id = get_suspension_id(new, 1, 0).to_owned();
        let second_suspension_id = get_suspension_id(new, 1, 1);

        *second_suspension_id = first_suspension_id;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn ocm_same_specimen() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, s1) = insert_test_suspension_and_specimen(&tx, |_| ())
            .await
            .unwrap();
        let shared_specimen_id = s1.record.specimen_id;
        let (_, s2) = insert_test_suspension_and_specimen(&tx, |new| {
            new.record.specimen_id = shared_specimen_id;
        })
        .await
        .unwrap();

        let (_, assay) = insert_test_chromium_assay(&tx).await.unwrap();
        let assay_id = assay.id;
        let person_id = s1.specimen.record.submitted_by;

        let new = NewChromiumRun::OnChipMultiplexing {
            common: new_common(assay_id, person_id),
            gem_wells: NonemptyBoundedVec::new(vec![NewOcmGemWell {
                readable_id: Uuid::new_v4().to_string().to_nonempty_string(),
                loading: NonemptyBoundedVec::new(vec![
                    NewOcmChipLoading::Suspension {
                        suspension_id: *s1.record.id,
                        common: loading_common(),
                        ocm_barcode_id: OcmBarcodeId::Ob1,
                    },
                    NewOcmChipLoading::Suspension {
                        suspension_id: *s2.record.id,
                        common: loading_common(),
                        ocm_barcode_id: OcmBarcodeId::Ob2,
                    },
                ])
                .unwrap(),
            }])
            .unwrap(),
        };

        let (_, run) = insert_test_ocm_chromium_run(&tx, |run| *run = new.clone())
            .await
            .unwrap();

        let detailed = select_chromium_run_by_id(&tx, *run.record.id)
            .await
            .unwrap();
        let gem_wells = detailed.gem_wells;

        assert_eq!(gem_wells.len(), 1);

        let specimens = &gem_wells[0].specimens;
        assert_eq!(specimens.len(), 2);

        assert_eq!(
            *specimens[0].specimen.record.id,
            *specimens[1].specimen.record.id
        );
        assert_ne!(specimens[0].ocm_barcode_id, specimens[1].ocm_barcode_id);
    }
}
