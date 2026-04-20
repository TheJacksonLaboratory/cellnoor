create type person_links as (
    self text,
    projects text,
    specimens text
);

-- The `email` field is nullable for the following situation:

-- John Doe signs up with email john.doe@jax.org
-- John Doe leaves The Jackson Laboratory
-- Another person named John Doe signs up. He now has the email "john.doe@jax.org"

-- In this situation, we still want to keep a record of the first John Doe, but that person just doesn't own the email
-- anymore. The first John Doe's email becomes `null`, with john.doe@jax.org now belonging to the new John Doe
create table if not exists person (
    id uuid primary key default uuidv7(),
    links person_links generated always as (('/people/' || id, '/people/' || id || '/projects', '/people/' || id || '/specimens')) stored not null,
    name case_insensitive_text not null,
    email case_insensitive_text unique,
    email_verified boolean not null default false,
    image text,
    created_at timestamptz not null,
    updated_at timestamptz not null,
    orcid case_insensitive_text unique
);

create table if not exists account (
    id uuid primary key default uuidv7(),
	person_id uuid references person on delete cascade on update cascade not null,
	organization_id uuid references organization on delete cascade on update cascade not null,
	account_id uuid not null,
	provider_id uuid not null,
	access_token text,
	refresh_token text,
	access_token_expires_at timestamptz,
	refresh_token_expires_at timestamptz,
	scope text,
	id_token text,
	created_at timestamptz not null,
	updated_at timestamptz not null
);

create table if not exists api_key (
    id uuid primary key default uuidv7(),
    config_id text not null,
    name text,
    start text,
    prefix text unique not null,
    hashed_key text not null,
    reference_id uuid not null,
    expires_t timestamptz not null,
    created_at timestamptz not null,
    updated_at timestamptz not null,
    permissions text not null
);
