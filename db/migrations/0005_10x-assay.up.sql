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
    library_types library_type[],
    sample_multiplexing sample_multiplexing,
    chemistry_version case_insensitive_text not null,
    protocol_url case_insensitive_text not null,
    chromium_chip case_insensitive_text,

    unique nulls not distinct (name, library_types, sample_multiplexing, chemistry_version, chromium_chip)
);

create function sort_assay_arrays() returns trigger language plpgsql volatile as $$
    begin
        new.library_types = array_sort(new.library_types::text[]);
        return new;
    end;
$$;

create trigger sort_assay_arrays before insert or update on tenx_assay for each row execute function
sort_assay_arrays();
