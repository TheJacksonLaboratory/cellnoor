create table project (
    id uuid primary key default uuidv7(),
    name case_insensitive_text unique not null,
    started_at timestamptz not null,
    ended_at timestamptz not null,

    constraint starts_before_ends check (started_at <= ended_at)
);

create table project_access (
    id uuid primary key default uuidv7(),
    project_id uuid references project on delete cascade not null,
    person_id uuid references person,
    api_key_id uuid references api_key,

    unique (project_id, person_id),
    unique (project_id, api_key_id),
    constraint has_person_or_api_key check ((person_id is null) != (api_key_id is null))
);
