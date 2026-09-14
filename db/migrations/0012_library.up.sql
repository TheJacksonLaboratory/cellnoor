create table library (
    id uuid primary key default uuidv7(),
    readable_id case_insensitive_text unique not null,
    cdna_id uuid not null,
    cdna_prepared_at timestamptz not null,
    single_index_set_name text references single_index_set,
    dual_index_set_name text references dual_index_set,
    number_of_sample_index_pcr_cycles integer not null,
    target_reads_per_cell integer,
    prepared_at timestamptz not null,
    additional_data jsonb,

    unique (id, prepared_at),
    foreign key (cdna_id, cdna_prepared_at) references cdna (id, prepared_at) on update cascade,

    constraint has_index check ((single_index_set_name is null) != (dual_index_set_name is null)),
    constraint prepared_after_cdna_prepared check (prepared_at >= cdna_prepared_at)
);

create function populate_library_cdna_timestamps() returns trigger language plpgsql as $$
    begin
        select prepared_at into new.cdna_prepared_at from cdna where id = new.cdna_id;

        return new;
    end;
$$;

create trigger library_cdna_timestamps before insert or update of cdna_id on library
for each row execute function populate_library_cdna_timestamps();

create table library_measurement (
    id uuid primary key default uuidv7(),
    library_id uuid not null,
    library_prepared_at timestamptz not null,
    measured_by uuid references person not null,
    measured_at timestamptz not null,
    data jsonb not null,

    unique (library_id, measured_by, measured_at, data),
    foreign key (library_id, library_prepared_at) references library (
        id, prepared_at
    ) on update cascade on delete cascade,

    constraint measured_after_library_prepared check (measured_at >= library_prepared_at)
);

create function populate_library_measurement_timestamps() returns trigger language plpgsql as $$
    begin
        select prepared_at into new.library_prepared_at from library where id = new.library_id;

        return new;
    end;
$$;

create trigger library_measurement_timestamps before insert or update of library_id on library_measurement
for each row execute function populate_library_measurement_timestamps();

create table library_preparer (
    library_id uuid references library on delete cascade not null,
    prepared_by uuid references person not null,
    primary key (library_id, prepared_by)
);
