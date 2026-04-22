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
create table person (
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

-- better-auth infects everything, so this table has to comply with https://better-auth.com/docs/concepts/database#account
create table person_account (
    id uuid primary key default uuidv7(),
	person_id uuid references person on delete cascade not null,
	organization_id uuid references organization on delete cascade not null,
	provider_id uuid not null,
	provider_user_id uuid not null,
	access_token text,
	refresh_token text,
	access_token_expires_at timestamptz,
	refresh_token_expires_at timestamptz,
	scope text,
	id_token text,
	created_at timestamptz not null,
	updated_at timestamptz not null
);

-- It would be nice to use better-auth's built-in utility for "organization-owned API keys", but it doesn't really work
-- with Postgres's row-level security
create table service_account (
    id uuid primary key default uuidv7(),
    owned_by_id uuid references person_account not null,
   	created_at timestamptz not null,
	updated_at timestamptz not null
);

-- Now, an API key can be owned by either a person or a service account. We don't use better-auth's system here because
-- it's a bit clunky for our usecase
create table api_key (
    id uuid primary key default uuidv7(),
    name text,
    description text,
    prefix text unique not null,
    hashed_key text not null,
    person_account_id uuid references person_account on delete cascade,
    service_account_id uuid references service_account on delete cascade,
    created_at timestamptz not null,
    updated_at timestamptz not null,
    expires_at timestamptz not null

    constraint has_account check ((person_account_id is null) != (service_account_id is null))
);
