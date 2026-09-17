use cellnoor_types::{
    Relation, cdna::creation::LibraryType, tenx_assay::creation::NewChromiumAssay,
};
use uuid::Uuid;

use crate::{
    db::{self, DbError, FieldValues, Insert},
    handlers::tenx_assays::create::{
        NewLibraryTypeSpecificationRecord, insert_library_type_specification,
    },
};

pub(super) async fn insert_chromium_assay(
    tx: &db::Transaction<'_>,
    assay: &NewChromiumAssay,
) -> Result<Uuid, DbError> {
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
) -> Result<Uuid, DbError> {
    tx.insert_returning_id(record).await
}

struct NewChromiumAssayRecord<'a> {
    inner: &'a NewChromiumAssay,
    library_types: Vec<LibraryType>,
}

impl Relation for NewChromiumAssayRecord<'_> {
    const NAME: &'static str = "tenx_assay";
}

impl Insert for NewChromiumAssayRecord<'_> {
    type Field = &'static str;

    fn fields(&self) -> FieldValues<'_, Self::Field> {
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

        vec![
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
    use cellnoor_types::{
        cdna::creation::LibraryType,
        nonempty::{NonemptyBoundedVec, NonemptyVec},
        positive::PositiveI32,
        tenx_assay::{
            SampleMultiplexing, TenxAssay,
            creation::{LibraryTypeSpecification, NewChromiumAssay, NewTenxAssay},
        },
    };
    use uuid::Uuid;

    use crate::{
        db::{self, DbError},
        handlers::{
            index_sets::insert_test_dual_index_set, tenx_assays::create::insert_tenx_assay,
        },
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    pub async fn insert_test_chromium_assay(
        tx: &db::Transaction<'_>,
    ) -> Result<(NewChromiumAssay, TenxAssay), DbError> {
        // The fixture's index set is well-formed, so only the database can
        // refuse it, and no test here reads that error
        let index_set_name = insert_test_dual_index_set(tx)
            .await
            .expect("failed to insert the test index set");
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
