pub mod measurement;

mod record {
    use jiff::Timestamp;
    use macro_attributes::base_model;
    use nonempty::NonemptyString;
    use positive::PositiveI32;
    use uuid::Uuid;

    use crate::tenx_assay::LibraryType;

    #[base_model]
    pub struct CdnaRecord<T> {
        pub id: T,
        pub readable_id: NonemptyString,
        pub library_type: LibraryType,
        pub prepared_at: Timestamp,
        pub gem_well_id: Option<Uuid>,
        pub n_amplification_cycles: PositiveI32,
        pub additional_data: Option<serde_json::Value>,
    }
}
