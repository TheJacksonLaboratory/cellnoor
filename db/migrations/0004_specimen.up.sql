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
