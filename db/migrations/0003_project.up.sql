-- Postgres skips a foreign key check when one of its columns is null. `ended_at` is nullable, so we group it with
-- `started_at` into a composite value, which is not null even when one of its fields is. When comparing two composite
-- values, Postgres treats two null fields as equal, so a specimen's copy of a project period with no end still matches
-- its project, and updates to the project cascade to it.
create type project_active_period as (started_at timestamptz, ended_at timestamptz);

create table project (
    id uuid primary key default uuidv7(),
    name case_insensitive_text unique not null,
    created_by uuid references principal not null default app_user_id(),
    active_period project_active_period not null,

    unique (id, active_period),
    constraint started_at_not_null check ((active_period).started_at is not null),
    constraint starts_before_ends check ((active_period).started_at <= (active_period).ended_at)
);

create table project_access (
    project_id uuid references project on delete cascade not null,
    principal_id uuid references principal on delete cascade not null,

    primary key (project_id, principal_id)
);
