create table suspension (
    id uuid primary key default uuidv7(),
    readable_id case_insensitive_text unique not null,
    specimen_id uuid references specimen not null,
    content case_insensitive_text not null,
    created_at timestamptz,
    lysis_duration_minutes real,
    target_cell_recovery bigint,
    additional_data jsonb
);

create table suspension_measurement (
    id uuid primary key default uuidv7(),
    suspension_id uuid references suspension on delete cascade not null,
    measured_by uuid references person not null,
    measured_at timestamptz not null,
    data jsonb not null,

    unique (suspension_id, measured_by, measured_at, data)
);

create table suspension_preparer (
    suspension_id uuid references suspension on delete cascade not null,
    prepared_by uuid references person not null,

    primary key (suspension_id, prepared_by)
);
