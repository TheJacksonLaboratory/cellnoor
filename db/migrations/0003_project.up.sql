create table project (
    id uuid primary key default uuidv7(),
    name case_insensitive_text unique not null,
    created_by_person uuid references person,
    created_by_service uuid references service,
    started_at timestamptz not null,
    ended_at timestamptz not null,

    constraint has_creator check ((created_by_person is null) != (created_by_service is null)),
    constraint starts_before_ends check (started_at <= ended_at)
);

create table project_access (
    id uuid primary key default uuidv7(),
    project_id uuid references project on delete cascade not null,
    person_id uuid references person,
    service_id uuid references service,

    unique (project_id, person_id),
    unique (project_id, service_id),
    constraint has_person_or_service check ((person_id is null) != (service_id is null))
);
