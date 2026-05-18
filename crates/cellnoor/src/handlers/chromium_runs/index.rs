use axum::{Json, extract::State};
use cellnoor_types::{
    chromium_run::{
        ChromiumRun, ChromiumRunField, ChromiumRunQuery, SavedChromiumRunRecord,
        SavedChromiumRunRecordDetailed, SavedGemWellWithSpecimensRecord,
    },
    order_by::OrderBy,
    tenx_assay::TenxAssay,
};
use deadpool_postgres::tokio_postgres::Row;
use futures::StreamExt;

use crate::{
    auth::AuthUser,
    db::{self, SqlTemplate},
    error::{Error, ErrorInner},
    state::AppState,
};

pub async fn index_chromium_runs(
    State(state): State<AppState>,
    user: AuthUser,
    Json(mut query): Json<ChromiumRunQuery>,
) -> Result<Json<Vec<ChromiumRun>>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_chromium_runs(&tx, &mut query).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn select_chromium_runs(
    tx: &db::Transaction<'_>,
    query: &mut ChromiumRunQuery,
) -> Result<Vec<ChromiumRun>, ErrorInner> {
    // The first column in the `order by` clause needs to match the `distinct on`
    // clause
    let distinct_on = OrderBy {
        field: ChromiumRunField::Id,
        desc: true,
    };

    query.order_by.push_front(distinct_on);

    let base_stmt = if query.detailed {
        include_str!("index/select_detailed.sql")
    } else {
        include_str!("index/select_compact.sql")
    };

    let sql = SqlTemplate::new(base_stmt).finish_with_query(query)?;

    let runs = if query.detailed {
        let stream = tx.query_stream(sql).await?;
        stream
            .map(|row| row.map(map_detailed_row).unwrap())
            .collect()
            .await
    } else {
        let stream = tx.query_stream_into(sql).await?;
        stream.map(ChromiumRun::from_record).collect().await
    };

    Ok(runs)
}

fn map_detailed_row(row: Row) -> ChromiumRun {
    let chromium_run: SavedChromiumRunRecord = row.get("chromium_run");
    let assay: TenxAssay = row.get("tenx_assay");
    let gem_wells: Vec<SavedGemWellWithSpecimensRecord> = row.get("gem_wells");

    ChromiumRun::from_detailed_record_and_gem_wells(
        SavedChromiumRunRecordDetailed {
            chromium_run,
            assay,
        },
        gem_wells,
    )
}

#[cfg(test)]
mod test {
    use cellnoor_types::{
        chromium_run::{
            ChromiumRun, ChromiumRunPredicateInner, ChromiumRunQuery,
            creation::{
                NewChromiumRun,
                ocm::{NewOcmChipLoading, NewOcmGemWell, OcmBarcodeId},
            },
        },
        operator::UuidOperator,
        suspension::Suspension,
    };
    use nonempty::NonemptyBoundedVec;
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use crate::{
        handlers::{
            chromium_runs::{
                create::{
                    insert_chromium_run,
                    test::{
                        insert_test_mixed_chromium_run, insert_test_ocm_chromium_run,
                        insert_test_standard_chromium_run, loading_common, new_common,
                    },
                },
                index::select_chromium_runs,
                show::select_chromium_run_by_id,
            },
            suspensions::create::test::insert_test_suspension_and_specimen,
            tenx_assays::create::insert_test_chromium_assay,
        },
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn select_compact() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, run) = insert_test_standard_chromium_run(&tx, |_| ())
            .await
            .unwrap();
        let id = *run.record().id;

        let mut query = ChromiumRunQuery::from_filter(
            ChromiumRunPredicateInner::Id(UuidOperator::Eq(id)).into(),
            false,
        );
        select_chromium_runs(&tx, &mut query).await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn select_detailed_standard() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, run) = insert_test_standard_chromium_run(&tx, |_| ())
            .await
            .unwrap();

        let ChromiumRun::Detailed { gem_wells, .. } =
            select_chromium_run_by_id(&tx, *run.record().id)
                .await
                .unwrap()
        else {
            unreachable!("expected ChromiumRun::Detailed");
        };

        assert_eq!(gem_wells.len(), 1);

        // Ensure that the two returned specimens are different and that they have
        // different multiplexing tags
        let specimens = &gem_wells[0].specimens;
        assert_eq!(specimens.len(), 2);
        assert_ne!(
            specimens[0].specimen.record().id,
            specimens[1].specimen.record().id
        );
        assert_ne!(
            specimens[0].multiplexing_tag.clone().unwrap(),
            specimens[1].multiplexing_tag.clone().unwrap()
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn select_detailed_ocm() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, run) = insert_test_ocm_chromium_run(&tx, |_| ()).await.unwrap();
        let ChromiumRun::Detailed { gem_wells, .. } =
            select_chromium_run_by_id(&tx, *run.record().id)
                .await
                .unwrap()
        else {
            unreachable!("expected ChromiumRun::Detailed");
        };

        assert_eq!(gem_wells.len(), 1);

        // Ensure that the two returned specimens are different and that they have
        // different OCM barcode IDs
        let specimens = &gem_wells[0].specimens;
        assert_eq!(specimens.len(), 2);
        assert_ne!(
            specimens[0].specimen.record().id,
            specimens[1].specimen.record().id
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
        let ChromiumRun::Detailed { gem_wells, .. } =
            select_chromium_run_by_id(&tx, *run.record().id)
                .await
                .unwrap()
        else {
            unreachable!("expected ChromiumRun::Detailed");
        };

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

        let ChromiumRun::Detailed { gem_wells, .. } =
            select_chromium_run_by_id(&tx, *run.record().id)
                .await
                .unwrap()
        else {
            unreachable!("expected ChromiumRun::Detailed");
        };

        assert_eq!(gem_wells.len(), 1);

        let specimens = &gem_wells[0].specimens;
        assert_eq!(specimens.len(), 2);

        assert_eq!(
            *specimens[0].specimen.record().id,
            *specimens[1].specimen.record().id
        );

        assert_ne!(specimens[0].ocm_barcode_id, specimens[1].ocm_barcode_id);
    }

    fn point_second_loading_at_first_suspension(new: &mut NewChromiumRun) {
        let NewChromiumRun::OnChipMultiplexing { gem_wells, .. } = new else {
            unreachable!("expected NewChromiumRun::OnChipMultiplexing");
        };
        let gem_well = &mut gem_wells.as_mut()[0];
        let loading = gem_well.loading.as_mut();

        let NewOcmChipLoading::Suspension {
            suspension_id: first_id,
            ..
        } = loading[0]
        else {
            unreachable!("expected NewOcmChipLoading::Suspension");
        };

        let NewOcmChipLoading::Suspension {
            suspension_id: second_id,
            ..
        } = &mut loading[1]
        else {
            unreachable!("expected NewOcmChipLoading::Suspension");
        };

        *second_id = first_id;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn ocm_same_specimen() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, s1) = insert_test_suspension_and_specimen(&tx, |_| ())
            .await
            .unwrap();
        let shared_specimen_id = s1.record().specimen_id;
        let (_, s2) = insert_test_suspension_and_specimen(&tx, |new| {
            new.record.specimen_id = shared_specimen_id;
        })
        .await
        .unwrap();

        let (_, assay) = insert_test_chromium_assay(&tx).await.unwrap();
        let assay_id = assay.id;
        let Suspension::Detailed { specimen, .. } = &s1 else {
            panic!("expected Suspension::Detailed");
        };
        let person_id = specimen.record().submitted_by;

        let new = NewChromiumRun::OnChipMultiplexing {
            common: new_common(assay_id, person_id),
            gem_wells: NonemptyBoundedVec::new(vec![NewOcmGemWell {
                readable_id: Uuid::new_v4().to_string().to_nonempty_string(),
                loading: NonemptyBoundedVec::new(vec![
                    NewOcmChipLoading::Suspension {
                        suspension_id: *s1.record().id,
                        common: loading_common(),
                        ocm_barcode_id: OcmBarcodeId::Ob1,
                    },
                    NewOcmChipLoading::Suspension {
                        suspension_id: *s2.record().id,
                        common: loading_common(),
                        ocm_barcode_id: OcmBarcodeId::Ob2,
                    },
                ])
                .unwrap(),
            }])
            .unwrap(),
        };

        let run = insert_chromium_run(&tx, new).await.unwrap();

        let ChromiumRun::Detailed { gem_wells, .. } =
            select_chromium_run_by_id(&tx, *run.record().id)
                .await
                .unwrap()
        else {
            unreachable!("expected ChromiumRun::Detailed");
        };

        assert_eq!(gem_wells.len(), 1);

        let specimens = &gem_wells[0].specimens;
        assert_eq!(specimens.len(), 2);

        assert_eq!(
            *specimens[0].specimen.record().id,
            *specimens[1].specimen.record().id
        );
        assert_ne!(specimens[0].ocm_barcode_id, specimens[1].ocm_barcode_id);
    }
}
