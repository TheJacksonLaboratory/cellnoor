create table suspension_pool (
    id uuid primary key default uuidv7(),
    readable_id case_insensitive_text unique not null,
    name case_insensitive_text not null,
    pooled_at timestamptz not null,
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

create trigger measurement_made_after_suspension_pool_created before insert or update on suspension_pool_measurement for each row execute function check_timestamp_ordering(
    'measured_at', 'pool_id', 'suspension_pool', 'pooled_at'
);

create table suspension_pool_preparer (
    pool_id uuid references suspension_pool on delete cascade not null,
    prepared_by uuid references person not null,

    primary key (pool_id, prepared_by)
);

create table multiplexing_tag (
    tag_id case_insensitive_text not null,
    type case_insensitive_text not null,

    primary key (tag_id, type)
);

create table suspension_pooling (
    id uuid primary key default uuidv7(),
    pool_id uuid references suspension_pool on delete cascade not null,
    suspension_id uuid references suspension on delete cascade not null,
    tag_id case_insensitive_text,
    tag_type case_insensitive_text,

    foreign key (tag_id, tag_type) references multiplexing_tag,
    unique (pool_id, tag_id, tag_type),
    unique nulls not distinct (pool_id, suspension_id, tag_id, tag_type)
);
