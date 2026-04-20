create type organization_links as (
    self text
);

create table if not exists organization (
    id uuid primary key,
    links organization_links generated always as (row('/organizations/' || id)) stored not null,
    name case_insensitive_text unique not null,
    microsoft_entra_tenant_id uuid unique not null
);
