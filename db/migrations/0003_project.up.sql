create table project (
    id uuid primary key default uuidv7(),
    name case_insensitive_text unique not null,
    created_by uuid references principal not null default app_user_id(),
    started_at timestamptz not null,
    ended_at timestamptz,

    unique (id, started_at, ended_at),
    constraint starts_before_ends check (started_at <= ended_at),
    constraint name_is_path_component check (name not in ('.', '..') and position('/' in name) = 0)
);

create table project_access (
    project_id uuid references project on delete cascade not null,
    principal_id uuid references principal on delete cascade not null,

    primary key (project_id, principal_id)
);
