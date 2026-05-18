use cellnoor_types::tenx_assay::{LibraryType, TenxAssay, creation::NewChromiumAssay};
use nonempty::NonemptyString;
use uuid::Uuid;

use crate::{
    db::{self, AsFieldValuePairs, FieldValuePairs, SqlTemplate, insert_into},
    error::ErrorInner,
    handlers::tenx_assays::create::{
        NewLibraryTypeSpecificationRecord, insert_library_type_specification,
    },
};

pub async fn insert_chromium_assay(
    tx: &db::Transaction<'_>,
    assay: &NewChromiumAssay,
) -> Result<Uuid, ErrorInner> {
    let library_types: Vec<_> = assay
        .library_type_specifications
        .iter()
        .map(|s| s.library_type)
        .collect();

    let record = NewChromiumAssayRecord {
        inner: assay,
        library_types,
    };

    let assay_id = insert_chromium_assay_record(tx, &record).await?;

    let lib_specs: Vec<_> = assay
        .library_type_specifications
        .iter()
        .map(|spec| NewLibraryTypeSpecificationRecord { assay_id, spec })
        .collect();

    futures::future::try_join_all(
        lib_specs
            .iter()
            .map(|spec| insert_library_type_specification(tx, spec)),
    )
    .await?;

    Ok(assay_id)
}

async fn insert_chromium_assay_record(
    tx: &db::Transaction<'_>,
    record: &NewChromiumAssayRecord<'_>,
) -> Result<Uuid, ErrorInner> {
    Ok(insert_into(tx, "tenx_assay", record).await?)
}

struct NewChromiumAssayRecord<'a> {
    inner: &'a NewChromiumAssay,
    library_types: Vec<LibraryType>,
}

impl AsFieldValuePairs<&'static str, 7> for NewChromiumAssayRecord<'_> {
    fn as_field_value_pairs(&self) -> FieldValuePairs<'_, &'static str, 7> {
        let Self {
            inner:
                NewChromiumAssay {
                    name,
                    chemistry_version,
                    protocol_url,
                    sample_multiplexing,
                    chromium_chip,
                    cmdlines,
                    library_type_specifications: _,
                },

            library_types,
        } = self;

        [
            ("name", name),
            ("chemistry_version", chemistry_version),
            ("protocol_url", protocol_url),
            ("sample_multiplexing", sample_multiplexing),
            ("chromium_chip", chromium_chip),
            ("cmdlines", cmdlines),
            ("library_types", library_types),
        ]
    }
}

#[cfg(test)]
pub mod tests {
    use cellnoor_types::tenx_assay::{
        LibraryType, SampleMultiplexing, TenxAssay,
        creation::{LibraryTypeSpecification, NewChromiumAssay, NewTenxAssay},
    };
    use nonempty::{NonemptyBoundedVec, NonemptyVec};
    use positive::PositiveI32;
    use uuid::Uuid;

    use crate::{
        db,
        error::ErrorInner,
        handlers::{
            index_sets::insert_test_dual_index_set,
            tenx_assays::create::{chromium::insert_chromium_assay, insert_tenx_assay},
        },
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    pub async fn insert_test_chromium_assay(
        tx: &db::Transaction<'_>,
    ) -> Result<(NewChromiumAssay, TenxAssay), ErrorInner> {
        let index_set_name = insert_test_dual_index_set(tx).await?;
        let kit_name = index_set_name[3..5].to_owned();

        let chromium_assay = NewChromiumAssay {
            name: Uuid::new_v4().to_string().to_nonempty_string(),
            chemistry_version: "v1".to_nonempty_string(),
            protocol_url: "https://10xgenomics.com".to_nonempty_string(),
            sample_multiplexing: SampleMultiplexing::Singleplex,
            chromium_chip: "GEM-X FX".to_nonempty_string(),
            cmdlines: NonemptyVec::new(vec!["cellranger count".to_nonempty_string()]).unwrap(),
            library_type_specifications: NonemptyBoundedVec::new(vec![LibraryTypeSpecification {
                library_type: LibraryType::GeneExpression,
                index_kit: kit_name,
                cdna_volume_µl: PositiveI32::new(50).unwrap(),
                library_volume_µl: PositiveI32::new(50).unwrap(),
            }])
            .unwrap(),
        };

        let assay = NewTenxAssay::Chromium(chromium_assay.clone());

        let inserted = insert_tenx_assay(tx, &assay).await?;

        Ok((chromium_assay, inserted))
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        insert_test_chromium_assay(&tx).await.unwrap();
    }
}
