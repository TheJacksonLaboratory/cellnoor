create table organization (
    id uuid primary key default uuidv7(),
    name case_insensitive_text unique not null,
    microsoft_entra_tenant_id uuid unique not null
);
