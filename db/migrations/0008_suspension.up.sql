create table suspension (
    id uuid primary key default uuidv7(),
    readable_id case_insensitive_text unique not null,
    specimen_id uuid not null,
    specimen_received_at timestamptz not null,
    content case_insensitive_text not null,
    created_at timestamptz,
    lysis_duration_minutes real,
    target_cell_recovery bigint,
    additional_data jsonb,

    unique (id, created_at),
    foreign key (specimen_id, specimen_received_at)
    references specimen (id, received_at) on update cascade,

    constraint only_nuclei_suspension_was_lysed check (content = 'nuclei' or lysis_duration_minutes is null),
    constraint created_after_specimen_received check (created_at >= specimen_received_at)
);

create function populate_suspension_specimen_timestamps() returns trigger language plpgsql as $$
    begin
        select received_at into new.specimen_received_at from specimen where id = new.specimen_id;

        return new;
    end;
$$;

create trigger suspension_specimen_timestamps before insert or update of specimen_id on suspension
for each row execute function populate_suspension_specimen_timestamps();

create table suspension_measurement (
    id uuid primary key default uuidv7(),
    -- `suspension.created_at` is nullable, and the default MATCH SIMPLE skips a foreign key check entirely when any of
    -- its columns is null. This plain reference keeps `suspension_id` validated in that case
    suspension_id uuid references suspension on delete cascade not null,
    suspension_created_at timestamptz,
    measured_by uuid references person not null,
    measured_at timestamptz not null,
    data jsonb not null,

    unique (suspension_id, measured_by, measured_at, data),
    foreign key (suspension_id, suspension_created_at)
    references suspension (id, created_at) on update cascade on delete cascade,

    constraint measured_after_suspension_created check (measured_at >= suspension_created_at)
);

create function populate_suspension_measurement_timestamps() returns trigger language plpgsql as $$
    begin
        select created_at into new.suspension_created_at from suspension where id = new.suspension_id;

        return new;
    end;
$$;

create trigger suspension_measurement_timestamps before insert or update of suspension_id on suspension_measurement
for each row execute function populate_suspension_measurement_timestamps();

create table suspension_preparer (
    suspension_id uuid references suspension on delete cascade not null,
    prepared_by uuid references person not null,

    primary key (suspension_id, prepared_by)
);
