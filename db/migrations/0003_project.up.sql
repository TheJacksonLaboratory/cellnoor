create type project_links as (
    self text,
    specimens text,
    chromium_datasets text
);

create table project (
    id uuid primary key default uuidv7(),
    links project_links generated always as (('/projects/' || id, '/projects/' || id || '/specimens', '/projects/' || id || '/chromium-datasets')) stored not null,
    name case_insensitive_text unique not null,
    started_at timestamptz not null,
    ended_at timestamptz not null,

    constraint starts_before_ends check (started_at < ended_at)
);

create table project_people (
    project_id uuid references project on delete cascade on update cascade not null,
    person_id uuid references people on delete cascade on update cascade not null,
    primary key (project_id, person_id)
);
