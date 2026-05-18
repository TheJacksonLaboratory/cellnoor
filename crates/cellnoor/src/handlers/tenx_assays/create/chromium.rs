use cellnoor_types::tenx_assay::{LibraryType, creation::NewChromiumAssay};
use uuid::Uuid;

use crate::{
    db::{self, AsFieldValuePairs, FieldValuePairs, insert_into},
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
