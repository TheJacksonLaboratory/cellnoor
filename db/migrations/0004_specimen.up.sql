-- As with `project_active_period`, the nullable `host_species` is grouped with `species` so that the foreign key to the
-- parent specimen is checked even when `host_species` is null
create type specimen_species as (species case_insensitive_text, host_species case_insensitive_text);

create table specimen (
    id uuid primary key default uuidv7(),
    readable_id case_insensitive_text unique not null,
    name case_insensitive_text not null,
    submitted_by uuid references person,
    project_id uuid not null,
    -- We denormalize the project's active period so we can constrain received_or_created_at
    project_active_period project_active_period not null,
    received_or_created_at timestamptz not null,
    parent_specimen_id uuid,
    parent_specimen_received_or_created_at timestamptz,
    species specimen_species not null,
    given_at timestamptz,
    given_by uuid references person,
    given_to uuid references person,
    type case_insensitive_text not null,
    embedded_in case_insensitive_text,
    fixative case_insensitive_text,
    thermal_preservation_method case_insensitive_text,
    tissue case_insensitive_text not null,
    additional_data jsonb,

    unique (id, received_or_created_at),
    unique (id, project_id, species, tissue),
    foreign key (project_id, project_active_period) references project (id, active_period) on update cascade,
    foreign key (parent_specimen_id, parent_specimen_received_or_created_at) references specimen (
        id, received_or_created_at
    ) match full on update cascade,
    foreign key (parent_specimen_id, project_id, species, tissue) references specimen (
        id, project_id, species, tissue
    ) on update cascade,

    constraint only_primary_has_submitter check ((parent_specimen_id is null) != (submitted_by is null)),

    constraint received_after_project_start check (received_or_created_at >= (project_active_period).started_at),
    constraint received_before_project_end check (received_or_created_at <= (project_active_period).ended_at),
    constraint received_before_given check (received_or_created_at < given_at),

    constraint created_after_parent check (parent_specimen_received_or_created_at < received_or_created_at),

    constraint given_is_fully_specified check (num_nulls(given_at, given_by, given_to) in (0, 3)),
    constraint species_not_null check ((species).species is not null),
    constraint host_species_different_from_donor_species check ((species).species != (species).host_species)
);

-- To prevent polluting application-code, we write a trigger that populates a specimens's project's start and end
create function populate_specimen_project_timestamps() returns trigger language plpgsql as $$
    begin
        select active_period into new.project_active_period from project where id = new.project_id;

        return new;
    end;
$$;

create trigger specimen_project_timestamps before insert or update of project_id on specimen for each row execute
function populate_specimen_project_timestamps();

create function populate_specimen_inherited_fields() returns trigger language plpgsql as $$
    begin
        if new.parent_specimen_id is null then
            new.parent_specimen_received_or_created_at := null;
            return new;
        end if;

        select
            received_or_created_at,
            project_id,
            project_active_period,
            species,
            tissue
        into
            new.parent_specimen_received_or_created_at,
            new.project_id,
            new.project_active_period,
            new.species,
            new.tissue
        from specimen where id = new.parent_specimen_id;

        return new;
    end;
$$;

create trigger specimen_inherited_fields before insert or update of parent_specimen_id on specimen for each row
execute function populate_specimen_inherited_fields();

create table committee_approval (
    institution_id uuid references institution on delete cascade not null,
    specimen_id uuid references specimen on delete cascade not null,
    committee_type case_insensitive_text not null,
    compliance_identifier case_insensitive_text not null,
    primary key (institution_id, specimen_id, committee_type)
);

create table specimen_measurement (
    id uuid primary key default uuidv7(),
    specimen_id uuid not null,
    specimen_received_or_created_at timestamptz not null,
    measured_by uuid references person not null,
    measured_at timestamptz not null,
    data jsonb not null,

    unique (specimen_id, measured_by, measured_at, data),
    foreign key (specimen_id, specimen_received_or_created_at) references specimen (
        id, received_or_created_at
    ) on update cascade on delete cascade,

    constraint measured_after_specimen_received check (measured_at >= specimen_received_or_created_at)
);

create function populate_specimen_measurement_timestamps() returns trigger language plpgsql as $$
    begin
        select received_or_created_at into new.specimen_received_or_created_at from specimen where id = new.specimen_id;

        return new;
    end;
$$;

create trigger specimen_measurement_timestamps before insert or update of specimen_id on specimen_measurement for each
row execute function populate_specimen_measurement_timestamps();
