-- A principal is anything that acts on the database: a person or a service. `app_user_id()` is always a principal's
-- id, and everything that records who did or owns something references a principal
create table principal (
    id uuid primary key,
    -- Staff bypass row-level security
    is_staff boolean not null default false
);

create table permission (
    principal_id uuid references principal on delete cascade not null,
    resource text not null,
    action text not null,

    primary key (principal_id, resource, action)
);

-- The `email` field is nullable for the following situation:

-- John Doe signs up with email john.doe@jax.org
-- John Doe leaves The Jackson Laboratory
-- Another person named John Doe signs up. He now has the email "john.doe@jax.org"

-- In this situation, we still want to keep a record of the first John Doe, but that person just doesn't own the email
-- anymore. The first John Doe's email becomes `null`, with john.doe@jax.org now belonging to the new John Doe
create table person (
    id uuid primary key default uuidv7() references principal on delete cascade,
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
create table account (
    id uuid primary key default uuidv7(),
    person_id uuid references person on delete cascade not null,
    auth_provider text not null,
    auth_provider_user_id text unique not null,
    scope text,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

-- It would be nice to use better-auth's built-in utility for "institution-owned API keys", but it doesn't really work
-- with Postgres's row-level security
create table service (
    id uuid primary key default uuidv7() references principal on delete cascade,
    description case_insensitive_text,
    owned_by uuid references person not null default app_user_id(),
    created_at timestamptz not null default now()
);

-- Every person and service is a principal, so a principal is created and dropped alongside them. These are
-- `security definer` so that the bookkeeping isn't subject to the policies on `principal`: whether the person or
-- service may be written at all has already been decided by that table's own policies
create function create_principal() returns trigger language plpgsql security definer as $$
    begin
        insert into principal (id) values (new.id);
        return new;
    end;
$$;

create function drop_principal() returns trigger language plpgsql security definer as $$
    begin
        delete from principal where id = old.id;
        return old;
    end;
$$;

create trigger create_principal before insert on person for each row execute function create_principal();
create trigger create_principal before insert on service for each row execute function create_principal();

create trigger drop_principal after delete on person for each row execute function drop_principal();
create trigger drop_principal after delete on service for each row execute function drop_principal();

create table service_access (
    service_id uuid references service on delete cascade not null,
    person_id uuid references person on delete cascade not null,

    primary key (service_id, person_id)
);

-- We don't use better-auth's API keys because they're a bit clunky for our usecase
create table api_key (
    id uuid primary key default uuidv7(),
    description case_insensitive_text,
    hashed_key bytea unique not null,
    owner_id uuid references principal on delete cascade not null,
    created_at timestamptz not null default now(),
    expires_at timestamptz,

    constraint created_before_expires check (created_at <= expires_at)
);
