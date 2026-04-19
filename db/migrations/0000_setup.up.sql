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

create or replace function create_user_if_not_exists(
    user_id text
) returns void language plpgsql volatile strict as $$
    begin
        perform create_role_if_not_exists(user_id);
        execute format('alter role %I with login password %L', user_id, uuidv7());
    end;
$$;


create or replace function create_user_with_password_from_file(
    user_id text, password_file_path text
) returns void language plpgsql volatile strict as $$
    begin
        perform create_user_if_not_exists(user_id);
        execute format('alter role %I with password %L', user_id, pg_read_file(password_file_path));
    end;
$$;

-- -- The app's connection pool is logged in as 'app_user'. Once the app grabs a connection and begins an operation, it
-- -- executed `set local role 'username'` to allow for permissions-checking
select create_user_with_password_from_file('app_user', '/run/secrets/app_user_password');

-- 'auth_user' creates users and API keys, but cannot do anything else
select create_user_with_password_from_file('auth_user', '/run/secrets/auth_user_password');

create collation case_insensitive (provider = icu, deterministic = false, locale = 'en-u-ks-level1');
create domain case_insensitive_text as text collate case_insensitive;
