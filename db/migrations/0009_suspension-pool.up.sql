create table suspension_pool (
    id uuid primary key default uuidv7(),
    readable_id case_insensitive_text unique not null,
    name case_insensitive_text not null,
    pooled_at timestamptz not null,
    additional_data jsonb,

    unique (id, pooled_at)
);

create table suspension_pool_measurement (
    id uuid primary key default uuidv7(),
    pool_id uuid not null,
    pool_created_at timestamptz not null,
    measured_by uuid references person not null,
    measured_at timestamptz not null,
    data jsonb not null,

    unique (pool_id, measured_by, measured_at, data),
    foreign key (pool_id, pool_created_at) references suspension_pool (id, pooled_at) on update cascade on delete cascade,

    constraint measured_after_pool_created check (measured_at >= pool_created_at)
);

create function populate_suspension_pool_measurement_timestamps() returns trigger language plpgsql as $$
    begin
        select pooled_at into new.pool_created_at from suspension_pool where id = new.pool_id;

        return new;
    end;
$$;

create trigger suspension_pool_measurement_timestamps before insert or update of pool_id
on suspension_pool_measurement
for each row execute function populate_suspension_pool_measurement_timestamps();

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
