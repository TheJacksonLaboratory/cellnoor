create table projects (
    id uuid primary key default uuidv7(),
    links jsonb generated always as (
        construct_links('projects', id, '{"people", "specimens", "chromium-datasets"}')
    ) stored not null,
    name case_insensitive_text unique not null,
    started_at timestamptz not null,
    ended_at timestamptz not null,

    constraint starts_before_ends check (started_at < ended_at)
);

create table project_people (
    project_id uuid references projects on delete restrict on update restrict not null,
    person_id uuid references people on delete restrict on update restrict not null,
    primary key (project_id, person_id)
);
