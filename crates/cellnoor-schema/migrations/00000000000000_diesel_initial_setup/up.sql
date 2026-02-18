-- This file was automatically created by Diesel to setup helper functions
-- and other internal bookkeeping. This file is safe to edit, any future
-- changes will be added to existing projects as new migrations.


-- Sets up a trigger for the given table to automatically set a column called
-- `updated_at` whenever the row is modified (unless `updated_at` was included
-- in the modified columns)
--
-- # Example
--
-- ```sql
-- CREATE TABLE users (id SERIAL PRIMARY KEY, updated_at TIMESTAMP NOT NULL DEFAULT NOW());
--
-- SELECT diesel_manage_updated_at('users');
-- ```

create or replace function diesel_manage_updated_at(_tbl regclass) returns void as $$
BEGIN
    EXECUTE format('CREATE TRIGGER set_updated_at BEFORE UPDATE ON %s
                    FOR EACH ROW EXECUTE PROCEDURE diesel_set_updated_at()', _tbl);
END;
$$ language plpgsql;

create or replace function diesel_set_updated_at() returns trigger as $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD AND
        NEW.updated_at IS NOT DISTINCT FROM OLD.updated_at
    ) THEN
        NEW.updated_at := current_timestamp;
    END IF;
    RETURN NEW;
END;
$$ language plpgsql;

create function role_exists(user_id text) returns boolean language plpgsql volatile strict as $$
    declare role_exists boolean;
    begin
        select exists (select 1 from pg_roles where rolname = user_id) into role_exists;
        return role_exists;
    end;
$$;

create function create_role_if_not_exists(
    role_name text
) returns void language plpgsql volatile strict as $$
    begin
        if not role_exists(role_name) then
            execute format('create role %I', role_name);
        end if;
    end;
$$;

create function construct_links(
    self_name text,
    id uuid,
    children text [] default '{}'
) returns jsonb language plpgsql immutable strict as $$
    declare links jsonb;
    declare child text;
    begin
        select json_object('self': concat('/', self_name, '/', id)) into links;
        foreach child in array children loop
            select links || json_object(child: concat('/', self_name, '/', id, '/', child))::jsonb into links;
        end loop;
        return links;
    end;
$$;

select create_role_if_not_exists('cellnoor_api');
alter role cellnoor_api with login;

select create_role_if_not_exists('cellnoor_ui');
alter role cellnoor_ui with login;

create collation case_insensitive (provider = icu, deterministic = false, locale = 'en-u-ks-level1');
create domain case_insensitive_text as text collate case_insensitive;

create function like_any(
    string text,
    patterns text []
) returns bool language plpgsql immutable strict as $$
    declare match bool;
    begin
        select string like any(patterns) into match;
        return match;
    end;
$$;
