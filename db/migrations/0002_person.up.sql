-- The `email` field is nullable for the following situation:

-- John Doe signs up with email john.doe@jax.org
-- John Doe leaves The Jackson Laboratory
-- Another person named John Doe signs up. He now has the email "john.doe@jax.org"

-- In this situation, we still want to keep a record of the first John Doe, but that person just doesn't own the email
-- anymore. The first John Doe's email becomes `null`, with john.doe@jax.org now belonging to the new John Doe
create table person (
    id uuid primary key default uuidv7(),
    name case_insensitive_text not null,
    email case_insensitive_text unique,
    email_verified boolean not null default false,
    institution_id uuid references institution not null,
    image text,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),
    orcid case_insensitive_text unique
);

-- better-auth infects everything, so this table has to comply with
-- https://better-auth.com/docs/concepts/database#account. Notice that we don't include any of the sensitive fields,
-- and we manually delete them in our auth configuration
create table person_account (
    id uuid primary key default uuidv7(),
    person_id uuid references person on delete cascade not null,
    -- Technically, `auth_provider_id` and `auth_provider_user_id` should be 'not null', but we leave them nullable so
    -- we can insert the admin user without knowing these things
    auth_provider_name text not null,
    auth_provider_user_id text not null,
    scope text,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

-- It would be nice to use better-auth's built-in utility for "institution-owned API keys", but it doesn't really work
-- with Postgres's row-level security
create table service_account (
    id uuid primary key default uuidv7(),
    name text not null,
    description text,
    owned_by uuid references person_account not null,
    created_at timestamptz not null
);

create table service_account_access (
    service_account_id uuid references service_account not null,
    person_id uuid references person not null,

    primary key (service_account_id, person_id)
);

create type hashed_api_key as (
    id uuid,
    person_id uuid,
    service_account_id uuid,
    hashed_key text,
    expires_at text
);

-- Now, an API key can be owned by either a person or a service account. We don't use better-auth's system here because
-- it's a bit clunky for our usecase
create table api_key (
    id uuid primary key default uuidv7(),
    name text,
    description text,
    prefix text unique not null,
    hashed_key text not null,
    person_id uuid references person on delete cascade,
    service_account_id uuid references service_account on delete cascade,
    created_at timestamptz not null,
    expires_at timestamptz not null

    constraint has_account check ((person_id is null) != (service_account_id is null)),
    constraint created_before_expires check (created_at < expires_at)
);
