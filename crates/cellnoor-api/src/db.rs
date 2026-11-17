pub use client::{Client, Pool, Transaction};
pub use error::DbError;
use postgres_types::ToSql;
pub use stmt::{FilterableSqlBuilder, Sql};

mod client;
#[cfg(feature = "dev")]
pub mod dummy_data;
mod error;
mod insert;
mod stmt;
#[cfg(test)]
pub mod test_utils;
mod update;

/// The fieldnames of a row paired with their values.
pub type FieldValues<'a, F> = Vec<(F, &'a (dyn ToSql + Sync))>;

/// [`FieldValues`] with every fieldname converted to a `&'static str`
type Columns<'a> = Vec<(&'static str, &'a (dyn ToSql + Sync))>;

/// [`Columns`] as the statement builders read it.
type ColumnSlice<'a> = [(&'static str, &'a (dyn ToSql + Sync))];

/// A type that
pub trait Insert: cellnoor_types::Relation {
    type Field: Into<&'static str>;

    fn fields(&self) -> FieldValues<'_, Self::Field>;
}

fn columns<T>(record: &T) -> Columns<'_>
where
    T: Insert,
{
    record
        .fields()
        .into_iter()
        .map(|(field, value)| (field.into(), value))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use cellnoor_types::{
        cdna::creation::LibraryType,
        chromium_run::creation::ocm::OcmBarcodeId,
        permission::{Action, Resource},
        specimen::{
            Fixative, Species, SpecimenType, ThermalPreservationMethod,
            creation::block::BlockEmbeddingMatrix,
        },
        suspension::SuspensionContent,
        suspension_pool::MultiplexingTagType,
        tenx_assay::SampleMultiplexing,
    };
    use postgres_types::{FromSqlOwned, ToSql};
    use strum::VariantArray;

    use crate::{db::Sql, state::dev_util::db_client_as_admin};

    async fn every_variant_round_trips<T>(enum_type: &str)
    where
        T: VariantArray + ToSql + FromSqlOwned + Sync + PartialEq + Debug,
    {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let variants = T::VARIANTS;
        let selected: Vec<T> = tx
            .query_one_into(&Sql::new(
                &format!("select $1::{enum_type}[]"),
                vec![&variants],
            ))
            .await
            .unwrap();

        pretty_assertions::assert_eq!(selected, variants);
    }

    #[tokio::test]
    async fn action() {
        every_variant_round_trips::<Action>("action").await;
    }

    #[tokio::test]
    async fn resource() {
        every_variant_round_trips::<Resource>("resource").await;
    }

    #[tokio::test]
    async fn species() {
        every_variant_round_trips::<Species>("species").await;
    }

    #[tokio::test]
    async fn specimen_type() {
        every_variant_round_trips::<SpecimenType>("specimen_type").await;
    }

    #[tokio::test]
    async fn block_embedding_matrix() {
        every_variant_round_trips::<BlockEmbeddingMatrix>("block_embedding_matrix").await;
    }

    #[tokio::test]
    async fn fixative() {
        every_variant_round_trips::<Fixative>("fixative").await;
    }

    #[tokio::test]
    async fn thermal_preservation_method() {
        every_variant_round_trips::<ThermalPreservationMethod>("thermal_preservation_method").await;
    }

    #[tokio::test]
    async fn library_type() {
        every_variant_round_trips::<LibraryType>("library_type").await;
    }

    #[tokio::test]
    async fn sample_multiplexing() {
        every_variant_round_trips::<SampleMultiplexing>("sample_multiplexing").await;
    }

    #[tokio::test]
    async fn suspension_content() {
        every_variant_round_trips::<SuspensionContent>("suspension_content").await;
    }

    #[tokio::test]
    async fn multiplexing_tag_type() {
        every_variant_round_trips::<MultiplexingTagType>("multiplexing_tag_type").await;
    }

    #[tokio::test]
    async fn ocm_barcode_id() {
        every_variant_round_trips::<OcmBarcodeId>("ocm_barcode_id").await;
    }
}
