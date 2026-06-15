create table suspension (
    id uuid primary key default uuidv7(),
    readable_id case_insensitive_text unique not null,
    specimen_id uuid references specimen not null,
    content case_insensitive_text not null,
    created_at timestamptz,
    lysis_duration_minutes real,
    target_cell_recovery bigint,
    additional_data jsonb,

    constraint only_nuclei_suspension_was_lysed check (content = 'nuclei' or lysis_duration_minutes is null)
);

create trigger suspension_created_after_specimen_received before insert or update on suspension for each row execute
function check_timestamp_ordering(
    'created_at', 'specimen_id', 'specimen', 'received_at'
);

create table suspension_measurement (
    id uuid primary key default uuidv7(),
    suspension_id uuid references suspension on delete cascade not null,
    measured_by uuid references person not null,
    measured_at timestamptz not null,
    data jsonb not null,

    unique (suspension_id, measured_by, measured_at, data)
);

create trigger measurement_made_after_suspension_created before insert or update on suspension_measurement
for each row execute function check_timestamp_ordering(
    'measured_at', 'suspension_id', 'suspension', 'created_at'
);

create table suspension_preparer (
    suspension_id uuid references suspension on delete cascade not null,
    prepared_by uuid references person not null,

    primary key (suspension_id, prepared_by)
);
