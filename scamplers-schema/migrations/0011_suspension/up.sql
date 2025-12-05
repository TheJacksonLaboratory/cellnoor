create table multiplexing_tags (
    id uuid primary key default uuidv7(),
    tag_id case_insensitive_text not null,
    type case_insensitive_text not null,

    unique (tag_id, type)
);

create table suspensions (
    id uuid primary key default uuidv7(),
    links jsonb generated always as (construct_links('suspensions', id, '{"measurements"}')) stored not null,
    readable_id case_insensitive_text unique not null,
    parent_specimen_id uuid references specimens on delete restrict on update restrict not null,
    content case_insensitive_text not null,
    created_at timestamptz,
    lysis_duration_minutes real,
    target_cell_recovery bigint not null,
    additional_data jsonb
);

create table suspension_tagging (
    suspension_id uuid references suspensions on delete restrict on update restrict not null,
    pool_id uuid references suspension_pools on delete restrict on update restrict not null,
    tag_id uuid references multiplexing_tags on delete restrict on update restrict not null,

    unique (pool_id, tag_id),
    primary key (suspension_id, pool_id, tag_id)
);

create table suspension_measurements (
    id uuid primary key default uuidv7(),
    suspension_id uuid references suspensions on delete restrict on update restrict not null,
    measured_by uuid references people on delete restrict on update restrict not null,
    measured_at timestamptz not null,
    data jsonb not null
);

create table suspension_preparers (
    suspension_id uuid references suspensions on delete restrict on update restrict not null,
    prepared_by uuid references people on delete restrict on update restrict not null,

    primary key (suspension_id, prepared_by)
);
