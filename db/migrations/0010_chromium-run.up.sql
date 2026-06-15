create table chromium_run (
    id uuid primary key default uuidv7(),
    readable_id case_insensitive_text unique not null,
    assay_id uuid references tenx_assay not null,
    run_at timestamptz not null,
    run_by uuid references person not null,
    succeeded boolean not null,
    additional_data jsonb
);

create table gem_well (
    id uuid primary key default uuidv7(),
    readable_id case_insensitive_text unique not null,
    chromium_run_id uuid references chromium_run on delete cascade not null
);

create table chip_loading (
    id uuid primary key default uuidv7(),
    gem_well_id uuid references gem_well on delete cascade not null,
    suspension_id uuid references suspension on delete cascade,
    suspension_pool_id uuid references suspension_pool on delete cascade,
    -- There are only 4 allowed OCM barcode IDs, but we let the application restrict this so there is only one source
    -- of truth (and so that we don't need a database migration if things change)
    ocm_barcode_id case_insensitive_text,

    unique nulls not distinct (gem_well_id, ocm_barcode_id),
    constraint has_suspension check ((suspension_id is null) != (suspension_pool_id is null))
);
