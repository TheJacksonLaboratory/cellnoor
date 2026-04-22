create table organization (
    id uuid primary key,
    links simple_links generated always as (row('/organizations/' || id)) stored not null,
    name case_insensitive_text unique not null,
    microsoft_entra_tenant_id uuid unique not null
);
