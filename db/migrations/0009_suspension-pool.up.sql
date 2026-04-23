create table suspension_pool (
    id uuid primary key default uuidv7(),
    links simple_links generated always as (row('/suspension-pools/' || id)) stored not null,
    readable_id case_insensitive_text unique not null,
    name case_insensitive_text not null,
    pooled_at timestamptz not null,
    multiplexing_type text not null,
    additional_data jsonb
);

create table suspension_pool_measurement (
    id uuid primary key default uuidv7(),
    pool_id uuid references suspension_pool on delete cascade not null,
    measured_by uuid references person not null,
    measured_at timestamptz not null,
    data jsonb not null,

    unique (pool_id, measured_by, measured_at, data)
);

create table suspension_pool_preparer (
    pool_id uuid references suspension_pool on delete cascade not null,
    prepared_by uuid references person not null,
    primary key (pool_id, prepared_by)
);

create table multiplexing_tag (
    id uuid primary key default uuidv7(),
    tag_id case_insensitive_text not null,
    type case_insensitive_text not null,

    unique (tag_id, type)
);

create table tagged_suspension_pooling (
    suspension_id uuid references suspension on delete cascade null,
    pool_id uuid references suspension_pool on delete cascade not null,
    tag_id uuid references multiplexing_tag not null,

    unique (pool_id, tag_id),
    primary key (suspension_id, pool_id, tag_id)
);

create table untagged_suspension_pooling (
    suspension_id uuid references suspension on delete cascade not null,
    pool_id uuid references suspension_pool on delete cascade not null,

    primary key (suspension_id, pool_id)
);
