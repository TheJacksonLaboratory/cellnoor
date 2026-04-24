create table chromium_run (
    id uuid primary key default uuidv7(),
    readable_id case_insensitive_text unique not null,
    assay_id uuid references tenx_assay not null,
    run_at timestamptz not null,
    run_by uuid references person not null,
    succeeded boolean not null,
    additional_data jsonb
);

create table gem_pool (
    id uuid primary key default uuidv7(),
    readable_id case_insensitive_text unique not null,
    chromium_run_id uuid references chromium_run on delete cascade not null
);

create table chip_loading (
    id uuid primary key default uuidv7(),
    gem_pool_id uuid references gem_pool on delete cascade not null,
    suspension_id uuid references suspension on delete cascade,
    -- There are only 4 allowed OCM barcode IDs, but we let the application restrict this so there is only one source of truth
    ocm_barcode_id case_insensitive_text,
    suspension_pool_id uuid references suspension_pool on delete cascade,
    suspension_volume_loaded jsonb not null,
    buffer_volume_loaded jsonb not null,
    additional_data jsonb,

    -- In theory, someone could insert two rows with the same `gem_pool_id` and `suspension_id` - one with an
    -- `ocm_barcode_id` and another without one, but the application prevents this
    unique nulls not distinct (gem_pool_id, suspension_id, ocm_barcode_id, suspension_pool_id),
    constraint has_suspension check ((suspension_id is null) != (suspension_pool_id is null))
);
