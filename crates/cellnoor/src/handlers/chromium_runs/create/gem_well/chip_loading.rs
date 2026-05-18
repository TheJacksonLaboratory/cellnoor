use cellnoor_types::chromium_run::creation::{
    NewChipLoadingCommonFields,
    ocm::{NewOcmChipLoading, OcmBarcodeId},
    standard::NewStandardChipLoading,
};
use uuid::Uuid;

use crate::{
    db::{self, AsFieldValuePairs, insert_into_no_returning},
    error::ErrorInner,
};

pub async fn insert_standard_chip_loading(
    tx: &db::Transaction<'_>,
    loading: &NewStandardChipLoading,
    gem_well_id: Uuid,
) -> Result<(), ErrorInner> {
    let chip_loading = NewChipLoadingRecord::from_standard_chip_loading(loading, gem_well_id);

    insert_chip_loading(tx, &chip_loading).await
}

pub async fn insert_ocm_chip_loading(
    tx: &db::Transaction<'_>,
    loading: &NewOcmChipLoading,
    gem_well_id: Uuid,
) -> Result<(), ErrorInner> {
    let chip_loading = NewChipLoadingRecord::from_ocm_chip_loading(loading, gem_well_id);

    insert_chip_loading(tx, &chip_loading).await
}

async fn insert_chip_loading(
    tx: &db::Transaction<'_>,
    chip_loading: &NewChipLoadingRecord<'_>,
) -> Result<(), ErrorInner> {
    Ok(insert_into_no_returning(tx, "chip_loading", chip_loading).await?)
}

#[derive(Clone, Debug, PartialEq)]
struct NewChipLoadingRecord<'a> {
    gem_well_id: Uuid,
    suspension_id: Option<Uuid>,
    suspension_pool_id: Option<Uuid>,
    ocm_barcode_id: Option<OcmBarcodeId>,
    common: &'a NewChipLoadingCommonFields,
}

impl<'a> NewChipLoadingRecord<'a> {
    fn from_standard_chip_loading(loading: &'a NewStandardChipLoading, gem_well_id: Uuid) -> Self {
        let (suspension_id, suspension_pool_id, common) = match loading {
            NewStandardChipLoading::Suspension {
                suspension_id,
                common,
            } => (Some(suspension_id), None, common),
            NewStandardChipLoading::SuspensionPool {
                suspension_pool_id,
                common,
            } => (None, Some(suspension_pool_id), common),
        };

        Self {
            gem_well_id,
            suspension_id: suspension_id.copied(),
            suspension_pool_id: suspension_pool_id.copied(),
            ocm_barcode_id: None,
            common,
        }
    }

    fn from_ocm_chip_loading(loading: &'a NewOcmChipLoading, gem_well_id: Uuid) -> Self {
        let (suspension_id, suspension_pool_id, common, ocm_barcode_id) = match loading {
            NewOcmChipLoading::Suspension {
                suspension_id,
                common,
                ocm_barcode_id,
            } => (Some(suspension_id), None, common, ocm_barcode_id),
            NewOcmChipLoading::SuspensionPool {
                suspension_pool_id,
                common,
                ocm_barcode_id,
            } => (None, Some(suspension_pool_id), common, ocm_barcode_id),
        };

        Self {
            gem_well_id,
            suspension_id: suspension_id.copied(),
            suspension_pool_id: suspension_pool_id.copied(),
            ocm_barcode_id: Some(*ocm_barcode_id),
            common,
        }
    }
}

impl<'a> AsFieldValuePairs<&'static str, 7> for NewChipLoadingRecord<'a> {
    fn as_field_value_pairs(&self) -> crate::db::FieldValuePairs<'_, &'static str, 7> {
        let Self {
            gem_well_id,
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

        [
            ("gem_well_id", gem_well_id),
            ("suspension_id", suspension_id),
            ("suspension_pool_id", suspension_pool_id),
            ("ocm_barcode_id", ocm_barcode_id),
            ("suspension_volume_loaded", suspension_volume_loaded),
            ("buffer_volume_loaded", buffer_volume_loaded),
            ("additional_data", additional_data),
        ]
    }
}
