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

#[cfg(any(test, feature = "dev"))]
pub struct TestChromiumAssayIds {
    pub flex: Uuid,
    pub ocm: Uuid,
    pub singleplex: Uuid,
}

#[cfg(any(test, feature = "dev"))]
pub async fn insert_chromium_assays(
    tx: &db::Transaction<'_>,
) -> Result<TestChromiumAssayIds, DbError> {
    use cellnoor_types::{
        nonempty::{NonemptyBoundedVec, NonemptyVec},
        positive::PositiveI32,
        tenx_assay::{
            SampleMultiplexing,
            creation::{LibraryTypeSpecification, NewTenxAssay},
        },
    };

    use crate::{
        db::Sql,
        handlers::{
            index_sets::{
                FLEX_DUAL_INDEX_SET_NAME, GENE_EXPRESSION_DUAL_INDEX_SET_NAME,
                insert_test_dual_index_sets,
            },
            tenx_assays::create::insert_tenx_assay,
        },
        state::dev_util::ToNonemptyString,
    };

    async fn select_or_insert(
        tx: &db::Transaction<'_>,
        assay: NewChromiumAssay,
    ) -> Result<Uuid, DbError> {
        static SELECT_ID: &str =
            "select id from tenx_assay where name = $1::case_insensitive_text and \
             sample_multiplexing = $2::case_insensitive_text and chemistry_version = \
             $3::case_insensitive_text and chromium_chip = $4::case_insensitive_text";

        let sql = Sql::new(
            SELECT_ID,
            vec![
                &assay.name,
                &assay.sample_multiplexing,
                &assay.chemistry_version,
                &assay.chromium_chip,
            ],
        );

        if let Some(&id) = tx.query_into::<Uuid>(&sql).await?.first() {
            return Ok(id);
        }

        Ok(insert_tenx_assay(tx, &NewTenxAssay::Chromium(assay))
            .await?
            .id)
    }

    insert_test_dual_index_sets(tx)
        .await
        .expect("failed to insert the test index set");

    tx.lock_table("tenx_assay").await?;

    let gene_expression = NonemptyBoundedVec::new(vec![LibraryTypeSpecification {
        library_type: LibraryType::GeneExpression,
        index_kit: GENE_EXPRESSION_DUAL_INDEX_SET_NAME[3..5].to_owned(),
        cdna_volume_µl: PositiveI32::new(40).unwrap(),
        library_volume_µl: PositiveI32::new(35).unwrap(),
    }])
    .unwrap();

    let singleplex = NewChromiumAssay {
        name: "Universal 3' Gene Expression".to_nonempty_string(),
        chemistry_version: "v4 - GEM-X".to_nonempty_string(),
        protocol_url: "https://www.10xgenomics.com/support/universal-three-prime-gene-expression/documentation/steps/library-prep/chromium-gem-x-single-cell-3-v4-gene-expression-user-guide".to_nonempty_string(),
        sample_multiplexing: SampleMultiplexing::Singleplex,
        chromium_chip: "GEM-X 3'".to_nonempty_string(),
        cmdlines: NonemptyVec::new(vec![
            "cellranger count".to_nonempty_string(),
            "cellranger multi".to_nonempty_string(),
        ])
        .unwrap(),
        library_type_specifications: gene_expression.clone(),
    };

    let flex = NewChromiumAssay {
        name: "Flex Gene Expression".to_nonempty_string(),
        chemistry_version: "v2 - GEM-X".to_nonempty_string(),
        protocol_url: "https://www.10xgenomics.com/support/flex-gene-expression/documentation/steps/library-prep/gem-x-flex-v2-for-multiplexed-samples".to_nonempty_string(),
        sample_multiplexing: SampleMultiplexing::FlexOligonucleotideBarcode,
        chromium_chip: "GEM-X FX".to_nonempty_string(),
        cmdlines: NonemptyVec::new(vec!["cellranger multi".to_nonempty_string()]).unwrap(),
        library_type_specifications: NonemptyBoundedVec::new(vec![LibraryTypeSpecification{library_type: LibraryType::GeneExpression, index_kit: FLEX_DUAL_INDEX_SET_NAME[3..5].to_owned(), cdna_volume_µl: PositiveI32::new(100).unwrap(), library_volume_µl: PositiveI32::new(40).unwrap()}]).unwrap(),
    };

    let ocm = NewChromiumAssay {
        name: "Universal 3' Gene Expression".to_nonempty_string(),
        chemistry_version: "v4 - GEM-X".to_nonempty_string(),
        protocol_url: "https://www.10xgenomics.com/support/universal-three-prime-gene-expression/documentation/steps/library-prep/gem-x-universal-3-prime-gene-expression-v-4-4-plex-reagent-kits".to_nonempty_string(),
        sample_multiplexing: SampleMultiplexing::OnChipMultiplexing,
        chromium_chip: "GEM-X OCM 3'".to_nonempty_string(),
        cmdlines: NonemptyVec::new(vec!["cellranger multi".to_nonempty_string()]).unwrap(),
        library_type_specifications: gene_expression,
    };

    Ok(TestChromiumAssayIds {
        flex: select_or_insert(tx, flex).await?,
        ocm: select_or_insert(tx, ocm).await?,
        singleplex: select_or_insert(tx, singleplex).await?,
    })
}

#[cfg(test)]
pub mod tests {

    use super::insert_chromium_assays;
    use crate::state::dev_util::db_client_as_admin;

    #[tokio::test(flavor = "multi_thread")]
    async fn insert() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        insert_chromium_assays(&tx).await.unwrap();
    }
}
