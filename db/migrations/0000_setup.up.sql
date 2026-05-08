create or replace function role_exists(role_name text) returns boolean language plpgsql volatile strict as $$
    declare role_exists boolean;
    begin
        select exists (select 1 from pg_roles where rolname = role_name) into role_exists;
        return role_exists;
    end;
$$;

create or replace function create_role_if_not_exists(
    role_name text
) returns void language plpgsql volatile strict as $$
    begin
        if not role_exists(role_name) then
            execute format('create role %I', role_name);
        end if;
    end;
$$;

-- We give the user a random, unguessable password so they could never sign in to the database directly
create or replace function create_user_if_not_exists(
    username text
) returns void language plpgsql volatile strict as $$
    begin
        perform create_role_if_not_exists(username);
        execute format('alter role %I with login password %L', username, uuidv7());
    end;
$$;


create or replace function create_user_with_password_from_file(
    username text, password_file_path text
) returns void language plpgsql volatile strict as $$
    begin
        perform create_user_if_not_exists(username);
        execute format('alter role %I with password %L', username, pg_read_file(password_file_path));
    end;
$$;

-- 'app' is the user as which the main application connects. Before executing a statement, it switches to the database
-- user representing the person (or API key)
select create_user_with_password_from_file('app', '/run/secrets/app_db_password');

-- 'auth' manages users and API keys, but cannot do anything else
select create_user_with_password_from_file('auth', '/run/secrets/auth_db_password');

-- Create a user who's an actual person, meaning they need some privileges
create or replace function create_person_user_if_not_exists(
    username text, is_staff boolean
) returns void language plpgsql volatile strict as $$
    begin
        perform create_user_if_not_exists(username);
        -- The nice thing here is that if a user already exists in the db, their staff-privilege will be set correctly no matter what
        if is_staff then
            -- Staff should be able to see everything
            execute format('alter user %I with bypassrls', username);
        else
            -- Demote a user back to row-level-security if they were previously staff
            execute format('alter user %I with nobypassrls', username);
        end if;
        -- These tables and views don't exist yet, but only these are granted to users so that a developer can't
        -- accidentally query against underlying tables, only views that have row-security policies enabled
        execute format('grant select on institution, person_public, project, project_access, project_detailed, specimen, specimen_detailed, suspension_to_specimen, suspension_pool_to_specimen, gem_pool_to_specimen, cdna_to_specimen, library_to_specimen, chromium_dataset_to_specimen to %I', username);
        -- The db user 'app' needs to be able to do `set role username`, but it shouldn't inherit that user's privileges
        execute format('grant %I to app with inherit false', username);
    end;
$$;

-- We might like to put a check-constraint here ensuring the string is non-empty, but our application has to do that
-- anyways for values in JSONB properties, so we do it there to avoid duplicating code
create collation case_insensitive (provider = icu, deterministic = false, locale = 'en-u-ks-level1');
create domain case_insensitive_text as text collate case_insensitive;

-- We want to insert nil UUIDs in a couple places, so we install this extension
create extension "uuid-ossp";
