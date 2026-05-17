use cellnoor_types::chromium_run::creation::{
    mixed::{NewMixedChipLoading, NewMixedGemPool},
    ocm::{MAX_SUSPENSIONS_PER_OCM_GEM_POOL, NewOcmGemPool},
    standard::NewStandardGemPool,
};
use uuid::Uuid;

use crate::{
    db::{self, AsFieldValuePairs, insert_into},
    error::ErrorInner,
    handlers::chromium_runs::create::gem_pool::chip_loading::{
        insert_ocm_chip_loading, insert_standard_chip_loading,
    },
};

mod chip_loading;

pub async fn insert_standard_gem_pool(
    tx: &db::Transaction<'_>,
    NewStandardGemPool {
        readable_id,
        loading,
    }: &NewStandardGemPool,
    chromium_run_id: Uuid,
) -> Result<(), ErrorInner> {
    let gem_pool = NewGemPoolRecord {
        readable_id: readable_id.as_ref(),
        chromium_run_id,
    };

    let gem_pool_id = insert_gem_pool(tx, &gem_pool).await?;
    insert_standard_chip_loading(tx, loading, gem_pool_id).await?;

    Ok(())
}

pub async fn insert_ocm_gem_pool(
    tx: &db::Transaction<'_>,
    NewOcmGemPool {
        readable_id,
        loading,
    }: &NewOcmGemPool,
    chromium_run_id: Uuid,
) -> Result<(), ErrorInner> {
    let gem_pool = NewGemPoolRecord {
        readable_id: readable_id.as_ref(),
        chromium_run_id,
    };

    let gem_pool_id = insert_gem_pool(tx, &gem_pool).await?;

    let chip_loading_insertions = loading
        .iter()
        .map(|l| insert_ocm_chip_loading(tx, l, gem_pool_id));
    futures::future::try_join_all(chip_loading_insertions).await?;

    Ok(())
}

pub async fn insert_mixed_gem_pool(
    tx: &db::Transaction<'_>,
    NewMixedGemPool {
        readable_id,
        loading,
    }: &NewMixedGemPool,
    chromium_run_id: Uuid,
) -> Result<(), ErrorInner> {
    let gem_pool = NewGemPoolRecord {
        readable_id: readable_id.as_ref(),
        chromium_run_id,
    };

    let gem_pool_id = insert_gem_pool(tx, &gem_pool).await?;

    let mut standard_chip_loading_insertions = Vec::with_capacity(MAX_SUSPENSIONS_PER_OCM_GEM_POOL);
    let mut ocm_chip_loading_insertions = Vec::with_capacity(MAX_SUSPENSIONS_PER_OCM_GEM_POOL);

    for l in loading {
        match l {
            NewMixedChipLoading::Standard(loading) => standard_chip_loading_insertions
                .push(insert_standard_chip_loading(tx, loading, gem_pool_id)),
            NewMixedChipLoading::Ocm(loading) => {
                ocm_chip_loading_insertions.push(insert_ocm_chip_loading(tx, loading, gem_pool_id))
            }
        }
    }

    tokio::try_join!(
        futures::future::try_join_all(standard_chip_loading_insertions),
        futures::future::try_join_all(ocm_chip_loading_insertions)
    )?;

    Ok(())
}

async fn insert_gem_pool(
    tx: &db::Transaction<'_>,
    gem_pool: &NewGemPoolRecord<'_>,
) -> Result<Uuid, ErrorInner> {
    Ok(insert_into(tx, "gem_pool", gem_pool).await?)
}

#[derive(Clone, Debug, PartialEq)]
struct NewGemPoolRecord<'a> {
    readable_id: &'a str,
    chromium_run_id: Uuid,
}

impl AsFieldValuePairs<&'static str, 2> for NewGemPoolRecord<'_> {
    fn as_field_value_pairs(&self) -> crate::db::FieldValuePairs<'_, &'static str, 2> {
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
