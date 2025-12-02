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

create table gems (
    id uuid primary key default uuidv7(),
    readable_id case_insensitive_text unique not null,
    chromium_run_id uuid not null references chromium_runs on delete restrict on update restrict
);

create table chip_loadings (
    id uuid primary key default uuidv7(),
    gems_id uuid references gems on delete restrict on update restrict not null,
    suspension_id uuid references suspensions on delete restrict on update restrict,
    suspension_pool_id uuid references suspension_pools on delete restrict on update restrict,
    suspension_volume_loaded jsonb not null,
    buffer_volume_loaded jsonb not null,
    additional_data jsonb,

    unique (gems_id, suspension_id),
    unique (gems_id, suspension_pool_id),
    constraint has_suspension check ((suspension_id is null) != (suspension_pool_id is null))
);
