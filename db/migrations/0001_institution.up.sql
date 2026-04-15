-- Table names are pluralized because it makes the Rust module have a different name from the modules in
-- `cellnoor-models`
create table institutions (
    id uuid primary key,
    links jsonb generated always as (construct_links('institutions', id, '{"members"}')) stored not null,
    name case_insensitive_text unique not null
);
