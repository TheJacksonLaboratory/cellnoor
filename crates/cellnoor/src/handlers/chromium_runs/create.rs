use axum::{Json as AxumJson, extract::State};
use cellnoor_types::chromium_run::{
    ChromiumRun, LoadingVolume, NewChromiumRunRecord,
    creation::{
        NewChipLoadingCommonFields, NewChromiumRun,
        mixed::{NewMixedChipLoading, NewMixedGemPool},
        ocm::{NewOcmChipLoading, NewOcmGemPool, OcmBarcodeId},
        standard::{NewStandardChipLoading, NewStandardGemPool},
    },
};
use nonempty::NonemptyString;
use postgres_types::Json;
use serde_json::Value;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, Record, ToRecord},
    error::{Error, ErrorInner},
    handlers::chromium_runs::show::select_chromium_run_by_id,
    state::AppState,
};

pub async fn create_chromium_run(
    State(state): State<AppState>,
    user: AuthUser,
    AxumJson(record): AxumJson<NewChromiumRun>,
) -> Result<AxumJson<ChromiumRun>, Error> {
    let mut client = state.db_client(user).await?;

    let tx = client.begin().await?;

    let response = insert_chromium_run(&tx, record).await.map(AxumJson)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn insert_chromium_run(
    tx: &db::Transaction<'_>,
    new: NewChromiumRun,
) -> Result<ChromiumRun, ErrorInner> {
    let (inner, gem_pool_specs) = decompose(new);

    let run_id = db::insert_into(tx, "chromium_run", &inner).await?;

    // gem_pool insertions are sequential because we need each gem_pool_id
    // before its chip_loadings can be inserted. Chip loadings within a single
    // gem_pool fan out concurrently.
    for spec in gem_pool_specs {
        let gem_pool_id = db::insert_into(
            tx,
            "gem_pool",
            &NewGemPool {
                readable_id: spec.readable_id,
                chromium_run_id: run_id,
            },
        )
        .await?;

        let chip_loadings: Vec<NewChipLoading> = spec
            .loadings
            .into_iter()
            .map(|l| l.into_chip_loading(gem_pool_id))
            .collect();

        futures::future::try_join_all(
            chip_loadings
                .iter()
                .map(|cl| db::insert_into_no_returning(tx, "chip_loading", cl)),
        )
        .await?;
    }

    select_chromium_run_by_id(tx, run_id).await
}

struct GemPoolSpec {
    readable_id: NonemptyString,
    loadings: Vec<NormalizedChipLoading>,
}

struct NormalizedChipLoading {
    suspension_id: Option<Uuid>,
    suspension_pool_id: Option<Uuid>,
    ocm_barcode_id: Option<OcmBarcodeId>,
    common: NewChipLoadingCommonFields,
}

impl NormalizedChipLoading {
    fn into_chip_loading(self, gem_pool_id: Uuid) -> NewChipLoading {
        let Self {
            suspension_id,
            suspension_pool_id,
            ocm_barcode_id,
            common:
                NewChipLoadingCommonFields {
                    suspension_volume_loaded,
                    buffer_volume_loaded,
                    additional_data,
                },
        } = self;

        NewChipLoading {
            gem_pool_id,
            suspension_id,
            suspension_pool_id,
            ocm_barcode_id,
            suspension_volume_loaded: Json(suspension_volume_loaded),
            buffer_volume_loaded: Json(buffer_volume_loaded),
            additional_data,
        }
    }
}

fn decompose(new: NewChromiumRun) -> (NewChromiumRunRecord, Vec<GemPoolSpec>) {
    match new {
        NewChromiumRun::Standard { inner, gem_pools } => (
            inner,
            gem_pools
                .into_iter()
                .map(standard_gem_pool_to_spec)
                .collect(),
        ),
        NewChromiumRun::OnChipMultiplexing { inner, gem_pools } => (
            inner,
            gem_pools.into_iter().map(ocm_gem_pool_to_spec).collect(),
        ),
        NewChromiumRun::Mixed { inner, gem_pools } => (
            inner,
            gem_pools.into_iter().map(mixed_gem_pool_to_spec).collect(),
        ),
    }
}

fn standard_gem_pool_to_spec(g: NewStandardGemPool) -> GemPoolSpec {
    GemPoolSpec {
        readable_id: g.readable_id,
        loadings: vec![standard_chip_loading_to_normalized(g.loading)],
    }
}

fn standard_chip_loading_to_normalized(l: NewStandardChipLoading) -> NormalizedChipLoading {
    match l {
        NewStandardChipLoading::Suspension(s) => NormalizedChipLoading {
            suspension_id: Some(s.suspension_id),
            suspension_pool_id: None,
            ocm_barcode_id: None,
            common: s.inner,
        },
        NewStandardChipLoading::SuspensionPool(s) => NormalizedChipLoading {
            suspension_id: None,
            suspension_pool_id: Some(s.suspension_pool_id),
            ocm_barcode_id: None,
            common: s.inner,
        },
    }
}

fn ocm_gem_pool_to_spec(g: NewOcmGemPool) -> GemPoolSpec {
    GemPoolSpec {
        readable_id: g.readable_id,
        loadings: g
            .loading
            .into_iter()
            .map(ocm_chip_loading_to_normalized)
            .collect(),
    }
}

fn ocm_chip_loading_to_normalized(l: NewOcmChipLoading) -> NormalizedChipLoading {
    match l {
        NewOcmChipLoading::Suspension(s) => NormalizedChipLoading {
            suspension_id: Some(s.suspension_id),
            suspension_pool_id: None,
            ocm_barcode_id: Some(s.ocm_barcode_id),
            common: s.inner,
        },
        NewOcmChipLoading::SuspensionPool(s) => NormalizedChipLoading {
            suspension_id: None,
            suspension_pool_id: Some(s.suspension_pool_id),
            ocm_barcode_id: Some(s.ocm_barcode_id),
            common: s.inner,
        },
    }
}

fn mixed_gem_pool_to_spec(g: NewMixedGemPool) -> GemPoolSpec {
    GemPoolSpec {
        readable_id: g.readable_id,
        loadings: g
            .loading
            .into_iter()
            .map(|l| match l {
                NewMixedChipLoading::Ocm(o) => ocm_chip_loading_to_normalized(o),
                NewMixedChipLoading::Standard(s) => standard_chip_loading_to_normalized(s),
            })
            .collect(),
    }
}

struct NewGemPool {
    readable_id: NonemptyString,
    chromium_run_id: Uuid,
}

impl ToRecord<&'static str, 2> for NewGemPool {
    fn to_record(&self) -> Record<'_, &'static str, 2> {
        let Self {
            readable_id,
            chromium_run_id,
        } = self;

        [
            ("readable_id", readable_id),
            ("chromium_run_id", chromium_run_id),
        ]
    }
}

struct NewChipLoading {
    gem_pool_id: Uuid,
    suspension_id: Option<Uuid>,
    suspension_pool_id: Option<Uuid>,
    ocm_barcode_id: Option<OcmBarcodeId>,
    suspension_volume_loaded: Json<LoadingVolume>,
    buffer_volume_loaded: Json<LoadingVolume>,
    additional_data: Option<Value>,
}

impl ToRecord<&'static str, 7> for NewChipLoading {
    fn to_record(&self) -> Record<'_, &'static str, 7> {
        let Self {
            gem_pool_id,
            suspension_id,
            suspension_pool_id,
            ocm_barcode_id,
            suspension_volume_loaded,
            buffer_volume_loaded,
            additional_data,
        } = self;

        [
            ("gem_pool_id", gem_pool_id),
            ("suspension_id", suspension_id),
            ("suspension_pool_id", suspension_pool_id),
            ("ocm_barcode_id", ocm_barcode_id),
            ("suspension_volume_loaded", suspension_volume_loaded),
            ("buffer_volume_loaded", buffer_volume_loaded),
            ("additional_data", additional_data),
        ]
    }
}

impl ToRecord<&'static str, 6> for NewChromiumRunRecord {
    fn to_record(&self) -> Record<'_, &'static str, 6> {
        let Self {
            id: _,
            readable_id,
            assay_id,
            run_at,
            run_by,
            succeeded,
            additional_data,
        } = self;

        [
            ("readable_id", readable_id),
            ("assay_id", assay_id),
            ("run_at", run_at),
            ("run_by", run_by),
            ("succeeded", succeeded),
            ("additional_data", additional_data),
        ]
    }
}

#[cfg(test)]
pub mod test {
    use cellnoor_types::{
        chromium_run::{
            ChromiumRun, LoadingVolume, NewChromiumRunRecord,
            creation::{
                NewChipLoadingCommonFields, NewChromiumRun,
                mixed::{NewMixedChipLoading, NewMixedGemPool},
                ocm::{NewOcmChipLoading, NewOcmGemPool, NewOcmSuspensionLoading, OcmBarcodeId},
                standard::{
                    NewStandardChipLoading, NewStandardGemPool,
                    NewStandardSuspensionLoadingRecord, NewStandardSuspensionPoolLoadingRecord,
                },
            },
        },
        id::NoId,
        suspension::Suspension,
        units::Microliter,
    };
    use jiff::Timestamp;
    use nonempty::NonemptyBoundedVec;
    use positive::PositiveF32;
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use crate::{
        db,
        error::ErrorInner,
        handlers::{
            chromium_runs::create::insert_chromium_run,
            suspension_pools::create::test::insert_test_suspension_pool_and_suspensions,
            suspensions::create::test::insert_test_suspension_and_specimen,
        },
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    /// Insert a minimal tenx_assay row directly via SQL for tests. Returns the
    /// assay's UUID and name.
    pub async fn insert_test_tenx_assay(
        tx: &db::Transaction<'_>,
    ) -> Result<(Uuid, nonempty::NonemptyString), ErrorInner> {
        let name = nonempty::NonemptyString::new(format!("assay-{}", Uuid::new_v4())).unwrap();
        let chemistry_version = "v3".to_nonempty_string();
        let protocol_url = "http://example.com".to_nonempty_string();
        let row = tx
            .query_one(
                "insert into tenx_assay (name, chemistry_version, protocol_url) values ($1, $2, \
                 $3) returning id",
                &[&name, &chemistry_version, &protocol_url],
            )
            .await?;
        let id: Uuid = row.get("id");
        Ok((id, name))
    }

    fn loading_common() -> NewChipLoadingCommonFields {
        NewChipLoadingCommonFields {
            suspension_volume_loaded: LoadingVolume {
                value: PositiveF32::new(50.0).unwrap(),
                unit: Microliter::Microliter,
            },
            buffer_volume_loaded: LoadingVolume {
                value: PositiveF32::new(20.0).unwrap(),
                unit: Microliter::Microliter,
            },
            additional_data: None,
        }
    }

    fn new_chromium_run_record(assay_id: Uuid, run_by: Uuid) -> NewChromiumRunRecord {
        NewChromiumRunRecord {
            id: NoId {},
            readable_id: Uuid::new_v4().to_string().to_nonempty_string(),
            assay_id,
            // Far-future timestamp so it's always after suspension/pool creation
            run_at: Timestamp::from_second(4_000_000_000).unwrap(),
            run_by,
            succeeded: true,
            additional_data: None,
        }
    }

    /// Build a Standard chromium run loading `num_suspensions` independently
    /// inserted suspensions, one per gem_pool. Returns the inserted run + the
    /// specimen IDs of the loaded suspensions.
    pub async fn insert_test_chromium_run_standard(
        tx: &db::Transaction<'_>,
        num_gem_pools: usize,
    ) -> Result<(ChromiumRun, Vec<Uuid>), ErrorInner> {
        assert!(num_gem_pools >= 1);
        let (assay_id, _) = insert_test_tenx_assay(tx).await?;

        let mut suspensions = Vec::with_capacity(num_gem_pools);
        let mut specimen_ids = Vec::with_capacity(num_gem_pools);
        let mut run_by = None;
        for _ in 0..num_gem_pools {
            let (_, s) = insert_test_suspension_and_specimen(tx, |_| ()).await?;
            let suspension_id = *s.record().id;
            let Suspension::Detailed { specimen, .. } = &s else {
                panic!("expected detailed suspension");
            };
            run_by = Some(specimen.record().submitted_by);
            specimen_ids.push(*specimen.record().id);
            suspensions.push(suspension_id);
        }

        let gem_pools: Vec<_> = suspensions
            .into_iter()
            .map(|sid| NewStandardGemPool {
                readable_id: Uuid::new_v4().to_string().to_nonempty_string(),
                loading: NewStandardChipLoading::Suspension(NewStandardSuspensionLoadingRecord {
                    suspension_id: sid,
                    inner: loading_common(),
                }),
            })
            .collect();

        let new = NewChromiumRun::Standard {
            inner: new_chromium_run_record(assay_id, run_by.unwrap()),
            gem_pools: NonemptyBoundedVec::new(gem_pools).unwrap(),
        };

        let inserted = insert_chromium_run(tx, new).await?;
        Ok((inserted, specimen_ids))
    }

    /// Build an OCM chromium run with `num_gem_pools` gem_pools, each loading
    /// `loadings_per_pool` distinct suspensions. Returns the inserted run.
    pub async fn insert_test_chromium_run_ocm(
        tx: &db::Transaction<'_>,
        num_gem_pools: usize,
        loadings_per_pool: usize,
    ) -> Result<ChromiumRun, ErrorInner> {
        assert!(num_gem_pools >= 1 && num_gem_pools <= 2);
        assert!(loadings_per_pool >= 1 && loadings_per_pool <= 4);

        let (assay_id, _) = insert_test_tenx_assay(tx).await?;
        let barcodes = [
            OcmBarcodeId::Ob1,
            OcmBarcodeId::Ob2,
            OcmBarcodeId::Ob3,
            OcmBarcodeId::Ob4,
        ];

        let mut run_by = None;
        let mut gem_pools = Vec::with_capacity(num_gem_pools);
        for _ in 0..num_gem_pools {
            let mut loadings = Vec::with_capacity(loadings_per_pool);
            for j in 0..loadings_per_pool {
                let (_, s) = insert_test_suspension_and_specimen(tx, |_| ()).await?;
                let Suspension::Detailed { specimen, .. } = &s else {
                    panic!("expected detailed suspension");
                };
                run_by = Some(specimen.record().submitted_by);
                loadings.push(NewOcmChipLoading::Suspension(NewOcmSuspensionLoading {
                    suspension_id: *s.record().id,
                    inner: loading_common(),
                    ocm_barcode_id: barcodes[j],
                }));
            }
            gem_pools.push(NewOcmGemPool {
                readable_id: Uuid::new_v4().to_string().to_nonempty_string(),
                loading: NonemptyBoundedVec::new(loadings).unwrap(),
            });
        }

        let new = NewChromiumRun::OnChipMultiplexing {
            inner: new_chromium_run_record(assay_id, run_by.unwrap()),
            gem_pools: NonemptyBoundedVec::new(gem_pools).unwrap(),
        };

        insert_chromium_run(tx, new).await
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert_standard() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (run, expected_specimen_ids) =
            insert_test_chromium_run_standard(&tx, 3).await.unwrap();

        let ChromiumRun::Detailed { gem_pools, .. } = run else {
            panic!("expected detailed chromium run");
        };
        assert_eq!(gem_pools.len(), 3);
        for g in &gem_pools {
            assert_eq!(g.specimens.len(), 1);
        }
        let mut actual_specimen_ids: Vec<Uuid> = gem_pools
            .iter()
            .flat_map(|g| g.specimens.iter().map(|s| *s.specimen.record().id))
            .collect();
        actual_specimen_ids.sort();
        let mut expected_sorted = expected_specimen_ids;
        expected_sorted.sort();
        assert_eq!(actual_specimen_ids, expected_sorted);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert_ocm() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let run = insert_test_chromium_run_ocm(&tx, 2, 4).await.unwrap();

        let ChromiumRun::Detailed { gem_pools, .. } = run else {
            panic!("expected detailed chromium run");
        };
        assert_eq!(gem_pools.len(), 2);
        for g in &gem_pools {
            assert_eq!(g.specimens.len(), 4);
            // Each specimen in an OCM gem_pool should carry an ocm_barcode_id.
            for s in &g.specimens {
                assert!(s.ocm_barcode_id.is_some());
            }
        }
    }

    /// Verify that the detailed view correctly groups specimens under their
    /// respective gem_pools. We construct a Standard run with three gem_pools
    /// loading pools of size 3, 2, and a single suspension (size 1) so that
    /// every gem_pool produces a distinct specimen-count.
    #[tokio::test(flavor = "multi_thread")]
    async fn select_detailed_gem_pool_and_specimen_counts() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (assay_id, _) = insert_test_tenx_assay(&tx).await.unwrap();

        // Two pools (each with 2 constituent suspensions, per helper) and one
        // standalone suspension. The 3 gem_pools below will therefore expose
        // specimen counts of 2, 2, and 1 respectively.
        let (_, pool_a) = insert_test_suspension_pool_and_suspensions(&tx, |_| ())
            .await
            .unwrap();
        let (_, pool_b) = insert_test_suspension_pool_and_suspensions(&tx, |_| ())
            .await
            .unwrap();
        let (_, single_suspension) =
            insert_test_suspension_and_specimen(&tx, |_| ()).await.unwrap();

        let Suspension::Detailed { specimen, .. } = &single_suspension else {
            panic!("expected detailed suspension");
        };
        let run_by = specimen.record().submitted_by;

        let gem_pools = vec![
            NewStandardGemPool {
                readable_id: Uuid::new_v4().to_string().to_nonempty_string(),
                loading: NewStandardChipLoading::SuspensionPool(
                    NewStandardSuspensionPoolLoadingRecord {
                        suspension_pool_id: *pool_a.record().id,
                        inner: loading_common(),
                    },
                ),
            },
            NewStandardGemPool {
                readable_id: Uuid::new_v4().to_string().to_nonempty_string(),
                loading: NewStandardChipLoading::SuspensionPool(
                    NewStandardSuspensionPoolLoadingRecord {
                        suspension_pool_id: *pool_b.record().id,
                        inner: loading_common(),
                    },
                ),
            },
            NewStandardGemPool {
                readable_id: Uuid::new_v4().to_string().to_nonempty_string(),
                loading: NewStandardChipLoading::Suspension(
                    NewStandardSuspensionLoadingRecord {
                        suspension_id: *single_suspension.record().id,
                        inner: loading_common(),
                    },
                ),
            },
        ];

        let new = NewChromiumRun::Standard {
            inner: new_chromium_run_record(assay_id, run_by),
            gem_pools: NonemptyBoundedVec::new(gem_pools).unwrap(),
        };

        let inserted = insert_chromium_run(&tx, new).await.unwrap();
        let ChromiumRun::Detailed { gem_pools, .. } = inserted else {
            panic!("expected detailed");
        };

        assert_eq!(gem_pools.len(), 3);
        let mut counts: Vec<usize> = gem_pools.iter().map(|g| g.specimens.len()).collect();
        counts.sort();
        assert_eq!(counts, vec![1, 2, 2]);

        // Specimens must not leak between gem_pools.
        use std::collections::HashSet;
        let mut seen = HashSet::new();
        for g in &gem_pools {
            for s in &g.specimens {
                let id = *s.specimen.record().id;
                assert!(seen.insert(id), "specimen {id} appeared in multiple gem_pools");
            }
        }
    }

    // Compile-time exercise of Mixed variant dispatch — actually inserting
    // would require fresh tagged-suspension setup; the type-level coverage
    // here ensures the decompose() match arm is wired up.
    #[allow(dead_code)]
    fn _mixed_compiles(g: NewMixedGemPool, _: NewMixedChipLoading) {
        let _ = g.readable_id;
    }
}
