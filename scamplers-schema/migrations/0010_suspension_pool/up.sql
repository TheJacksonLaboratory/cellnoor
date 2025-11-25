create table suspension_pools (
    id uuid primary key default uuidv7(),
    links jsonb generated always as (
        construct_links('suspension-pools', id, '{"measurements", "suspensions"}')
    ) stored not null,
    readable_id case_insensitive_text unique not null,
    name case_insensitive_text not null,
    pooled_at timestamptz not null,
    additional_data jsonb
);

create table suspension_pool_measurements (
    id uuid primary key default uuidv7(),
    pool_id uuid references suspension_pools on delete restrict on update restrict not null,
    measured_by uuid references people on delete restrict on update restrict not null,
    measured_at timestamptz not null,
    data jsonb not null
);

create table suspension_pool_preparers (
    pool_id uuid references suspension_pools on delete restrict on update restrict not null,
    prepared_by uuid references people on delete restrict on update restrict not null,
    primary key (pool_id, prepared_by)
);
