use cellnoor_types::{
    Relation,
    chromium_run::creation::{
        LoadedEntity,
        ocm::{OcmBarcodeId, OcmLoadedEntity},
    },
};
use uuid::Uuid;

use crate::db::{self, DbError, FieldValues, Insert};

pub(super) async fn insert_standard_chip_loading(
    tx: &db::Transaction<'_>,
    loading: &LoadedEntity,
    gem_well_id: Uuid,
) -> Result<(), DbError> {
    let chip_loading = NewChipLoadingRecord::from_standard_chip_loading(loading, gem_well_id);

    insert_chip_loading(tx, &chip_loading).await
}

pub(super) async fn insert_ocm_chip_loading(
    tx: &db::Transaction<'_>,
    loading: &OcmLoadedEntity,
    gem_well_id: Uuid,
) -> Result<(), DbError> {
    let chip_loading = NewChipLoadingRecord::from_ocm_chip_loading(loading, gem_well_id);

    insert_chip_loading(tx, &chip_loading).await
}

async fn insert_chip_loading(
    tx: &db::Transaction<'_>,
    chip_loading: &NewChipLoadingRecord,
) -> Result<(), DbError> {
    tx.insert(chip_loading).await
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

impl Relation for NewChipLoadingRecord {
    const NAME: &'static str = "chip_loading";
}

impl Insert for NewChipLoadingRecord {
    type Field = &'static str;

    fn fields(&self) -> FieldValues<'_, Self::Field> {
        let Self {
            gem_well_id,
            suspension_id,
            suspension_pool_id,
            ocm_barcode_id,
        } = self;

        vec![
            ("gem_well_id", gem_well_id),
            ("suspension_id", suspension_id),
            ("suspension_pool_id", suspension_pool_id),
            ("ocm_barcode_id", ocm_barcode_id),
        ]
    }
}
