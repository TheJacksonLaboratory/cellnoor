create table library_type_specification (
    assay_id uuid references tenx_assay on delete cascade not null,
    library_type case_insensitive_text not null,
    index_kit case_insensitive_text references index_kit on delete cascade not null,
    cdna_volume_µl integer not null,
    library_volume_µl integer not null,
    primary key (assay_id, library_type)
);
