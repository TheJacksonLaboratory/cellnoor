create table chromium_runs (
    id uuid primary key default uuidv7(),
    readable_id case_insensitive_text unique not null,
    links jsonb generated always as (construct_links('chromium-runs', id)) stored not null,
    assay_id uuid references tenx_assays on delete restrict on update restrict not null,
    run_at timestamptz not null,
    run_by uuid references people on delete restrict on update restrict not null,
    succeeded boolean not null,
    additional_data jsonb
);

create table gem_pools (
    id uuid primary key default uuidv7(),
    links jsonb generated always as (construct_links('gem-pools', id)) stored not null,
    readable_id case_insensitive_text unique not null,
    chromium_run_id uuid not null references chromium_runs on delete restrict on update restrict
);

create table chip_loadings (
    id uuid primary key default uuidv7(),
    gem_pool_id uuid references gem_pools on delete restrict on update restrict not null,
    suspension_id uuid references suspensions on delete restrict on update restrict,
    ocm_barcode_id case_insensitive_text,
    suspension_pool_id uuid references suspension_pools on delete restrict on update restrict,
    suspension_volume_loaded jsonb not null,
    buffer_volume_loaded jsonb not null,
    additional_data jsonb,

    -- In theory, someone could insert two rows with the same `gem_pool_id` and `suspension_id` - one with an
    -- `ocm_barcode_id` and another without one, but the application prevents this
    unique nulls not distinct (gem_pool_id, suspension_id, ocm_barcode_id),
    unique nulls not distinct (gem_pool_id, suspension_pool_id, ocm_barcode_id),
    constraint has_suspension check ((suspension_id is null) != (suspension_pool_id is null))
);
