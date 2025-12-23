-- The `email` field is nullable for the following situation:

-- John Doe signs up with email john.doe@jax.org
-- John Doe leaves The Jackson Laboratory
-- Another person named John Doe signs up. He now has the email john.doe@jax.org

-- In this situation, the first John Doe should have his email become `null`, with john.doe@jax.org now belonging to
-- the new John Doe
create table people (
    id uuid primary key default uuidv7(),
    links jsonb generated always as (
        construct_links('people', id, '{"labs", "specimens", "chromium-datasets"}')
    ) stored not null,
    name case_insensitive_text not null,
    email case_insensitive_text unique,
    email_verified boolean not null default false,
    institution_id uuid references institutions on delete restrict on update restrict not null,
    orcid case_insensitive_text unique,
    microsoft_entra_oid uuid unique
);

create table api_keys (
    prefix bytea unique not null,
    created_at timestamptz not null default current_timestamp,
    hash text unique not null,
    user_id uuid references people on delete cascade on update cascade not null,
    primary key (prefix, hash)
);
