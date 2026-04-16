#[cfg(feature = "app")]
use cellnoor_schema::chip_loadings;
use macro_attributes::{base_model, insert, simple_enum};
use macros::impl_enum_to_sql;
use non_empty::NonEmptyVec;
#[cfg(feature = "app")]
use schemars::JsonSchema;
use uuid::Uuid;

use crate::chromium_run::common::{ChipLoadingFields, GemPoolFields};
#[cfg(feature = "app")]
use crate::utils::EnumToSql;

pub const MAX_SUSPENSIONS_PER_OCM_GEM_POOL: usize = 4;

#[simple_enum]
#[derive(strum::VariantArray)]
pub enum OcmBarcodeId {
    Ob1,
    Ob2,
    Ob3,
    Ob4,
}

#[cfg(feature = "app")]
impl EnumToSql for OcmBarcodeId {}
impl_enum_to_sql!(OcmBarcodeId);

#[insert]
#[cfg_attr(feature = "app", diesel(table_name = chip_loadings), schemars(inline))]
pub struct SuspensionLoading {
    pub suspension_id: Uuid,
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    pub inner: ChipLoadingFields,
    pub ocm_barcode_id: OcmBarcodeId,
}

impl SuspensionLoading {
    #[must_use]
    pub fn suspension_id(&self) -> Uuid {
        self.suspension_id
    }
}

#[insert]
#[cfg_attr(feature = "app", diesel(table_name = chip_loadings), schemars(inline))]
pub struct SuspensionPoolLoading {
    pub suspension_pool_id: Uuid,
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    pub inner: ChipLoadingFields,
    pub ocm_barcode_id: OcmBarcodeId,
}

impl SuspensionPoolLoading {
    #[must_use]
    pub fn suspension_pool_id(&self) -> Uuid {
        self.suspension_pool_id
    }
}

#[base_model]
#[derive(serde::Deserialize)]
#[cfg_attr(feature = "app", derive(JsonSchema))]
#[serde(untagged, rename_all = "snake_case")]
pub enum OcmChipLoading {
    Suspension(SuspensionLoading),
    SuspensionPool(SuspensionPoolLoading),
}

impl OcmChipLoading {
    #[must_use]
    pub fn suspension_id(&self) -> Option<Uuid> {
        let Self::Suspension(l) = self else {
            return None;
        };

        Some(l.suspension_id())
    }

    #[must_use]
    pub fn suspension_pool_id(&self) -> Option<Uuid> {
        let Self::SuspensionPool(l) = self else {
            return None;
        };

        Some(l.suspension_pool_id())
    }
}

#[base_model]
#[derive(serde::Deserialize)]
#[cfg_attr(feature = "app", derive(JsonSchema))]
pub struct OcmGemPool {
    #[serde(flatten)]
    pub inner: GemPoolFields,
    pub loading: NonEmptyVec<OcmChipLoading, MAX_SUSPENSIONS_PER_OCM_GEM_POOL>,
}

impl OcmGemPool {
    fn loading(&self) -> &[OcmChipLoading] {
        self.loading.as_ref()
    }

    pub fn suspension_ids(&self) -> Vec<Uuid> {
        self.loading()
            .iter()
            .filter_map(OcmChipLoading::suspension_id)
            .collect()
    }

    pub fn suspension_pool_ids(&self) -> Vec<Uuid> {
        self.loading()
            .iter()
            .filter_map(OcmChipLoading::suspension_pool_id)
            .collect()
    }
}
