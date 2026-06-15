use cellnoor_types::chromium_run::creation::{
    LoadedEntity,
    ocm::{OcmBarcodeId, OcmLoadedEntity},
};
use uuid::Uuid;

use crate::{
    db::{self, AsFieldValuePairs, insert_into_no_returning},
    error::ErrorInner,
};

pub(super) async fn insert_standard_chip_loading(
    tx: &db::Transaction<'_>,
    loading: &LoadedEntity,
    gem_well_id: Uuid,
) -> Result<(), ErrorInner> {
    let chip_loading = NewChipLoadingRecord::from_standard_chip_loading(loading, gem_well_id);

    insert_chip_loading(tx, &chip_loading).await
}

pub(super) async fn insert_ocm_chip_loading(
    tx: &db::Transaction<'_>,
    loading: &OcmLoadedEntity,
    gem_well_id: Uuid,
) -> Result<(), ErrorInner> {
    let chip_loading = NewChipLoadingRecord::from_ocm_chip_loading(loading, gem_well_id);

    insert_chip_loading(tx, &chip_loading).await
}

async fn insert_chip_loading(
    tx: &db::Transaction<'_>,
    chip_loading: &NewChipLoadingRecord,
) -> Result<(), ErrorInner> {
    Ok(insert_into_no_returning(tx, "chip_loading", chip_loading).await?)
}

#[derive(Clone, Debug, PartialEq)]
struct NewChipLoadingRecord {
    gem_well_id: Uuid,
    suspension_id: Option<Uuid>,
    suspension_pool_id: Option<Uuid>,
    ocm_barcode_id: Option<OcmBarcodeId>,
}

impl NewChipLoadingRecord {
    fn from_standard_chip_loading(loaded_entity: &LoadedEntity, gem_well_id: Uuid) -> Self {
        let (suspension_id, suspension_pool_id) = match loaded_entity {
            LoadedEntity::Suspension { suspension_id } => (Some(suspension_id), None),
            LoadedEntity::SuspensionPool { suspension_pool_id } => (None, Some(suspension_pool_id)),
        };

        Self {
            gem_well_id,
            suspension_id: suspension_id.copied(),
            suspension_pool_id: suspension_pool_id.copied(),
            ocm_barcode_id: None,
        }
    }

    fn from_ocm_chip_loading(
        OcmLoadedEntity {
            loaded_entity,
            ocm_barcode_id,
        }: &OcmLoadedEntity,
        gem_well_id: Uuid,
    ) -> Self {
        let (suspension_id, suspension_pool_id) = match loaded_entity {
            LoadedEntity::Suspension { suspension_id } => (Some(suspension_id), None),
            LoadedEntity::SuspensionPool { suspension_pool_id } => (None, Some(suspension_pool_id)),
        };

        Self {
            gem_well_id,
            suspension_id: suspension_id.copied(),
            suspension_pool_id: suspension_pool_id.copied(),
            ocm_barcode_id: Some(*ocm_barcode_id),
        }
    }
}

impl AsFieldValuePairs<&'static str, 4> for NewChipLoadingRecord {
    fn as_field_value_pairs(&self) -> crate::db::FieldValuePairs<'_, &'static str, 4> {
        let Self {
            gem_well_id,
            suspension_id,
            suspension_pool_id,
            ocm_barcode_id,
        } = self;

        [
            ("gem_well_id", gem_well_id),
            ("suspension_id", suspension_id),
            ("suspension_pool_id", suspension_pool_id),
            ("ocm_barcode_id", ocm_barcode_id),
        ]
    }
}
