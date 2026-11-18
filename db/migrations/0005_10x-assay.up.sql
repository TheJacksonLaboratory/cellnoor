create type library_type as enum (
    'Antibody Capture',
    'Antigen Capture',
    'Chromatin Accessibility',
    'CRISPR Guide Capture',
    'Custom',
    'Gene Expression',
    'Multiplexing Capture',
    'VDJ',
    'VDJ B',
    'VDJ T',
    'VDJ T GD'
);

create type sample_multiplexing as enum (
    'cellplex',
    'flex_barcode',
    'flex_oligonucleotide_barcode',
    'hashtag',
    'on_chip_multiplexing',
    'singleplex'
);

create table tenx_assay (
    id uuid primary key default uuidv7(),
    name case_insensitive_text not null,
    sample_multiplexing sample_multiplexing,
    chemistry_version case_insensitive_text not null,
    protocol_url case_insensitive_text not null,
    chromium_chip case_insensitive_text,

    unique nulls not distinct (name, sample_multiplexing, chemistry_version, chromium_chip)
);
