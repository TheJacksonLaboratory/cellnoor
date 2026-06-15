create table specimen (
    id uuid primary key default uuidv7(),
    readable_id case_insensitive_text unique not null,
    name case_insensitive_text not null,
    submitted_by uuid references person not null,
    project_id uuid references project not null,
    received_at timestamptz not null,
    species case_insensitive_text not null,
    host_species case_insensitive_text,
    returned_at timestamptz,
    returned_by uuid references person,
    type case_insensitive_text not null,
    embedded_in case_insensitive_text,
    fixative case_insensitive_text,
    thermal_preservation_method case_insensitive_text,
    tissue case_insensitive_text not null,
    additional_data jsonb,

    constraint received_before_returned check (received_at < returned_at),
    constraint host_species_different_from_donor_species check (species != host_species)
);

-- In order to ensure a timestamp on a child entity makes sense relative to its parent, we define a trigger function
-- that works for any table
create function check_timestamp_ordering() returns trigger language plpgsql volatile strict as $$
    declare
        child_value_field text = tg_argv[0];
        child_fk_field text = tg_argv[1];
        parent_table text = tg_argv[2];
        parent_value_field text = tg_argv[3];
        new_json jsonb = to_jsonb(new);
        child_value timestamptz = (new_json ->> child_value_field)::timestamptz;
        child_fk uuid = (new_json ->> child_fk_field)::uuid;
        n integer;
    begin
        if (child_value is null) then
            return new;
        end if;

        execute format('select count(*) from %I where id = $1 and (%I <= $2 or %I is null)', parent_table, parent_value_field, parent_value_field) into n using child_fk, child_value;

        if (n != 1) then
            raise check_violation using message = format('%I cannot be before parent %I field %I', child_value_field, parent_table, parent_value_field), table = tg_table_name, column = child_value_field;
        end if;

        return new;
    end;
$$;

create trigger check_specimen_received_after_project_started before insert or update on specimen for each row execute
function check_timestamp_ordering(
    'received_at', 'project_id', 'project', 'started_at'
);


create table committee_approval (
    institution_id uuid references institution on delete cascade not null,
    specimen_id uuid references specimen on delete cascade not null,
    committee_type case_insensitive_text not null,
    compliance_identifier case_insensitive_text not null,
    primary key (institution_id, specimen_id, committee_type)
);

create table specimen_measurement (
    id uuid primary key default uuidv7(),
    specimen_id uuid references specimen on delete cascade not null,
    measured_by uuid references person not null,
    measured_at timestamptz not null,
    data jsonb not null,

    unique (specimen_id, measured_by, measured_at, data)
);

create trigger specimen_before_measurement before insert or update on specimen_measurement for each row execute
function check_timestamp_ordering(
    'measured_at', 'specimen_id', 'specimen', 'received_at'
);
