create table specimens (
    id uuid primary key default uuidv7(),
    readable_id case_insensitive_text unique not null,
    links jsonb generated always as (
        construct_links('specimens', id, '{"measurements", "suspensions", "chromium-datasets"}')
    ) stored not null,
    name case_insensitive_text not null,
    submitted_by uuid references people on delete restrict on update restrict not null,
    lab_id uuid references labs on delete restrict on update restrict not null,
    received_at timestamptz not null,
    species case_insensitive_text not null,
    host_species case_insensitive_text,
    returned_at timestamptz,
    returned_by uuid references people on delete restrict on update restrict,
    type case_insensitive_text not null,
    embedded_in case_insensitive_text,
    fixative case_insensitive_text,
    frozen bool not null default false,
    cryopreserved bool not null default false,
    tissue case_insensitive_text not null,
    additional_data jsonb,

    constraint not_both_frozen_and_cryopreserved check (not (cryopreserved and frozen))
);

create table committee_approval (
    institution_id uuid references institutions on delete restrict on update restrict not null,
    specimen_id uuid references specimens on delete restrict on update restrict not null,
    committee_type case_insensitive_text not null,
    compliance_identifier case_insensitive_text not null,
    primary key (institution_id, committee_type, specimen_id)
);

create table specimen_measurements (
    id uuid primary key default uuidv7(),
    specimen_id uuid not null references specimens on delete restrict on update restrict,
    measured_by uuid not null references people on delete restrict on update restrict,
    measured_at timestamptz not null,
    data jsonb not null
);
