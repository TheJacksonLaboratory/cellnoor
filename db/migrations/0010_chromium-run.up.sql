create table chromium_run (
    id uuid primary key default uuidv7(),
    readable_id case_insensitive_text unique not null,
    assay_id uuid references tenx_assay not null,
    run_at timestamptz not null,
    run_by uuid references person not null,
    succeeded boolean not null,
    additional_data jsonb,

    unique (id, run_at)
);

-- A GEM well has no timestamp of its own, so we denormalize its run's. This is what lets cDNA prepared from this GEM
-- well be constrained to being after the run.
create table gem_well (
    id uuid primary key default uuidv7(),
    readable_id case_insensitive_text unique not null,
    chromium_run_id uuid not null,
    run_at timestamptz not null,

    unique (id, run_at),
    foreign key (chromium_run_id, run_at) references chromium_run (id, run_at) on update cascade on delete cascade
);

create function populate_gem_well_chromium_run_timestamps() returns trigger language plpgsql as $$
    begin
        select run_at into new.run_at from chromium_run where id = new.chromium_run_id;

        return new;
    end;
$$;

create trigger gem_well_chromium_run_timestamps before insert or update of chromium_run_id on gem_well
for each row execute function populate_gem_well_chromium_run_timestamps();

create table chip_loading (
    id uuid primary key default uuidv7(),
    gem_well_id uuid not null,
    run_at timestamptz not null,
    -- Both parents' timestamps are not null, so each pair below is null together or not at all. `match full` enforces
    -- that, which means each composite foreign key validates its own id column
    suspension_id uuid,
    suspension_created_at timestamptz,
    suspension_pool_id uuid,
    suspension_pool_pooled_at timestamptz,
    -- There are only 4 allowed OCM barcode IDs, but we let the application restrict this so there is only one source
    -- of truth (and so that we don't need a database migration if things change)
    ocm_barcode_id case_insensitive_text,

    unique nulls not distinct (gem_well_id, ocm_barcode_id),
    foreign key (gem_well_id, run_at) references gem_well (id, run_at) on update cascade on delete cascade,
    foreign key (suspension_id, suspension_created_at) references suspension (
        id, created_at
    ) match full on update cascade on delete cascade,
    foreign key (suspension_pool_id, suspension_pool_pooled_at) references suspension_pool (
        id, pooled_at
    ) match full on update cascade on delete cascade,

    constraint has_suspension check ((suspension_id is null) != (suspension_pool_id is null)),
    constraint run_after_suspension_created check (run_at >= suspension_created_at),
    constraint run_after_suspension_pool_pooled check (run_at >= suspension_pool_pooled_at)
);

create function populate_chip_loading_timestamps() returns trigger language plpgsql as $$
    begin
        select run_at into new.run_at from gem_well where id = new.gem_well_id;
        select created_at into new.suspension_created_at from suspension where id = new.suspension_id;
        select pooled_at into new.suspension_pool_pooled_at from suspension_pool where id = new.suspension_pool_id;

        return new;
    end;
$$;

create trigger chip_loading_timestamps
before insert or update of gem_well_id, suspension_id, suspension_pool_id on chip_loading
for each row execute function populate_chip_loading_timestamps();
