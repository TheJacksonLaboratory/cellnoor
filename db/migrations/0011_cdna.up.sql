create table cdna (
    id uuid primary key default uuidv7(),
    readable_id case_insensitive_text unique not null,
    library_type case_insensitive_text not null,
    prepared_at timestamptz not null,
    gem_well_id uuid,
    -- We denormalize the GEM well's Chromium run time so we can constrain prepared_at to being after it. `match full`
    -- requires this to be null exactly when `gem_well_id` is null, which is what keeps `gem_well_id` validated.
    gem_well_run_at timestamptz,
    n_amplification_cycles integer,
    additional_data jsonb,

    -- a single GEM well cannot generate more than one cDNA of the same library type
    unique (gem_well_id, library_type),
    unique (id, prepared_at),
    foreign key (gem_well_id, gem_well_run_at) references gem_well (id, run_at) match full on update cascade,

    constraint prepared_after_chromium_run check (prepared_at >= gem_well_run_at)
);

create function populate_cdna_gem_well_timestamps() returns trigger language plpgsql as $$
    begin
        select run_at into new.gem_well_run_at from gem_well where id = new.gem_well_id;

        return new;
    end;
$$;

create trigger cdna_gem_well_timestamps before insert or update of gem_well_id on cdna
for each row execute function populate_cdna_gem_well_timestamps();

create table cdna_measurement (
    id uuid primary key default uuidv7(),
    cdna_id uuid not null,
    cdna_prepared_at timestamptz not null,
    measured_by uuid references person not null,
    measured_at timestamptz not null,
    data jsonb not null,

    unique (cdna_id, measured_by, measured_at, data),
    foreign key (cdna_id, cdna_prepared_at) references cdna (id, prepared_at) on update cascade on delete cascade,

    constraint measured_after_cdna_prepared check (measured_at >= cdna_prepared_at)
);

create function populate_cdna_measurement_timestamps() returns trigger language plpgsql as $$
    begin
        select prepared_at into new.cdna_prepared_at from cdna where id = new.cdna_id;

        return new;
    end;
$$;

create trigger cdna_measurement_timestamps before insert or update of cdna_id on cdna_measurement
for each row execute function populate_cdna_measurement_timestamps();

create table cdna_preparer (
    cdna_id uuid references cdna on delete cascade not null,
    prepared_by uuid references person not null,
    primary key (cdna_id, prepared_by)
);
