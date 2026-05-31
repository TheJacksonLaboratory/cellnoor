use cellnoor_types::chromium_run::creation::{
    mixed::{NewMixedChipLoading, NewMixedGemWell},
    ocm::{MAX_SUSPENSIONS_PER_OCM_GEM_WELL, NewOcmGemWell},
    standard::NewStandardGemWell,
};
use nonempty::NonemptyString;
use uuid::Uuid;

use crate::{
    db::{self, AsFieldValuePairs, insert_into},
    error::ErrorInner,
    handlers::chromium_runs::create::gem_well::chip_loading::{
        insert_ocm_chip_loading, insert_standard_chip_loading,
    },
};

mod chip_loading;

pub(super) async fn insert_standard_gem_well(
    tx: &db::Transaction<'_>,
    NewStandardGemWell {
        readable_id,
        loading,
    }: &NewStandardGemWell,
    chromium_run_id: Uuid,
) -> Result<(), ErrorInner> {
    let gem_well = NewGemWellRecord {
        readable_id,
        chromium_run_id,
    };

    let gem_well_id = insert_gem_well(tx, &gem_well).await?;
    insert_standard_chip_loading(tx, loading, gem_well_id).await?;

    Ok(())
}

pub(super) async fn insert_ocm_gem_well(
    tx: &db::Transaction<'_>,
    NewOcmGemWell {
        readable_id,
        loading,
    }: &NewOcmGemWell,
    chromium_run_id: Uuid,
) -> Result<(), ErrorInner> {
    let gem_well = NewGemWellRecord {
        readable_id,
        chromium_run_id,
    };

    let gem_well_id = insert_gem_well(tx, &gem_well).await?;

    let chip_loading_insertions = loading
        .iter()
        .map(|l| insert_ocm_chip_loading(tx, l, gem_well_id));
    futures::future::try_join_all(chip_loading_insertions).await?;

    Ok(())
}

pub(super) async fn insert_mixed_gem_well(
    tx: &db::Transaction<'_>,
    NewMixedGemWell {
        readable_id,
        loading,
    }: &NewMixedGemWell,
    chromium_run_id: Uuid,
) -> Result<(), ErrorInner> {
    let gem_well = NewGemWellRecord {
        readable_id,
        chromium_run_id,
    };

    let gem_well_id = insert_gem_well(tx, &gem_well).await?;

    let mut standard_chip_loading_insertions = Vec::with_capacity(MAX_SUSPENSIONS_PER_OCM_GEM_WELL);
    let mut ocm_chip_loading_insertions = Vec::with_capacity(MAX_SUSPENSIONS_PER_OCM_GEM_WELL);

    match loading {
        NewMixedChipLoading::Standard(loading) => standard_chip_loading_insertions
            .push(insert_standard_chip_loading(tx, loading, gem_well_id)),
        NewMixedChipLoading::Ocm(loadings) => {
            for l in loadings {
                ocm_chip_loading_insertions.push(insert_ocm_chip_loading(tx, l, gem_well_id))
            }
        }
    }

    tokio::try_join!(
        futures::future::try_join_all(standard_chip_loading_insertions),
        futures::future::try_join_all(ocm_chip_loading_insertions)
    )?;

    Ok(())
}

async fn insert_gem_well(
    tx: &db::Transaction<'_>,
    gem_well: &NewGemWellRecord<'_>,
) -> Result<Uuid, ErrorInner> {
    Ok(insert_into(tx, "gem_well", gem_well).await?)
}

#[derive(Clone, Debug, PartialEq)]
struct NewGemWellRecord<'a> {
    readable_id: &'a NonemptyString,
    chromium_run_id: Uuid,
}

impl AsFieldValuePairs<&'static str, 2> for NewGemWellRecord<'_> {
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
