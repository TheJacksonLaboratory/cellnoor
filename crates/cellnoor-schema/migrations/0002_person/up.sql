-- The `email` field is nullable for the following situation:

-- John Doe signs up with email john.doe@jax.org
-- John Doe leaves The Jackson Laboratory
-- Another person named John Doe signs up. He now has the email john.doe@jax.org

-- In this situation, the first John Doe should have his email become `null`, with john.doe@jax.org now belonging to
-- the new John Doe
create table people (
    id uuid primary key default uuidv7(),
    links jsonb generated always as (
        construct_links('people', id, '{"projects", "specimens", "chromium-datasets"}')
    ) stored not null,
    name case_insensitive_text not null,
    email case_insensitive_text unique,
    email_verified boolean not null default false,
    institution_id uuid references institutions on delete restrict on update restrict not null,
    orcid case_insensitive_text unique,
    microsoft_entra_oid uuid unique,
    is_admin boolean not null default false,
    is_biology_staff boolean not null default false,
    is_computational_staff boolean not null default false
);

create table json_web_keys (
    id uuid primary key default uuidv7(),
    public_key text not null,
    private_key text not null,
    created_at timestamptz not null,
    expires_at timestamptz not null
);

-- This table is just for displaying the JWTs a user has created in the UI. It's not used for auth checks in
-- cellnoor-api
create table json_web_tokens (
    jti uuid primary key,
    sub uuid references people on delete cascade on update cascade,
    name case_insensitive_text not null,
    description case_insensitive_text,
    iat timestamptz not null,
    exp timestamptz not null
);

create table revoked_json_web_tokens (
    jti uuid primary key,
    exp timestamptz not null
);
