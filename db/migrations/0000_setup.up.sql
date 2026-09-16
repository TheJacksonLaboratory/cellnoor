-- This is used in row-level security to determine who the current user is. The app sets the `app.user_id` setting in
-- crates/cellnoor/src/db/client.rs
create function app_user_id() returns uuid language sql stable as $$
    select current_setting('app.user_id', true)::uuid
$$;

create function create_user_with_password_from_file(
    username text, password_file_path text
) returns void language plpgsql volatile strict as $$
    begin
        if not exists (select 1 from pg_roles where rolname = username) then
            execute format('create role %I with login', username);
        end if;
        execute format('alter role %I with password %L', username, pg_read_file(password_file_path));
    end;
$$;

-- 'app' is the user as which the main application connects. It is subject to row-level security on every table
select create_user_with_password_from_file('app', '/run/secrets/app_db_password');

-- 'auth' manages people and accounts on behalf of the authentication service, but cannot do anything else
select create_user_with_password_from_file('auth', '/run/secrets/auth_db_password');

-- We might like to put a check-constraint here ensuring the string is non-empty, but our application has to do that
-- anyways for values in JSONB properties, so we do it there to avoid duplicating code
create collation case_insensitive (provider = icu, deterministic = false, locale = 'en-u-ks-level1');
create domain case_insensitive_text as text collate case_insensitive;

create extension pg_trgm;
-- We want to insert nil UUIDs in a couple places, so we install this extension
create extension "uuid-ossp";
