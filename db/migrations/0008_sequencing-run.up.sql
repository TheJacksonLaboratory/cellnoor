create table sequencing_runs (
    id uuid primary key default uuidv7(),
    links jsonb generated always as (construct_links('sequencing-runs', id, '{"libraries"}')) stored not null,
    readable_id case_insensitive_text unique not null,
    begun_at timestamptz not null,
    finished_at timestamptz,
    additional_data jsonb,

    constraint begun_before_finished check (begun_at <= finished_at)
);
