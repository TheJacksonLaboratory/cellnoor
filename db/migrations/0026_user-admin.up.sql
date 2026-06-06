alter user app with bypassrls;

-- 'app' needs to be able to read API keys directly to authenticate people
grant select on api_key to app;

-- In order for permission_grantor to give permissions to other roles, it needs permissions itself
grant all on all tables in schema public to user_creator with grant option;

-- It also needs to create roles
alter user user_creator createrole;
grant create on schema public to user_creator;

create or replace function user_can_admin_users(user_id uuid) returns boolean language plpgsql volatile strict as $$
    declare can_admin boolean;
    begin
        select can_admin_users from person where id = user_id into can_admin;
        return can_admin;
    end;
$$;

create or replace function user_is_person(user_id uuid) returns boolean language plpgsql volatile strict as $$
    declare is_person boolean;
    begin
        select exists (select 1 from person where id = user_id) into is_person;
        return is_person;
    end;
$$;

create type permission_set as (
    actions text,
    tableset text
);

set role user_creator;

create or replace function _validate_user_can_admin_users(
    user_id uuid
) returns void language plpgsql volatile strict as $$
    begin
        if not user_can_admin_users(user_id) then
            raise insufficient_privilege using message = 'user cannot admin other users';
        end if;
    end;
$$;

create or replace function _validate_user_is_person(user_id uuid) returns void language plpgsql volatile strict as $$
    begin
        if not user_is_person(user_id) then
            raise insufficient_privilege using message = 'user is not person';
        end if;
    end;
$$;


create or replace function _grant_permissions_to_person_as_user_creator(
    current_user_id uuid, target_user_id uuid, permissions permission_set []
) returns void language plpgsql volatile strict security definer as $$
    declare
        user_is_person boolean;
        perm permission_set;
    begin
        perform _validate_user_can_admin_users(current_user_id);
        perform _validate_user_is_person(target_user_id);

        foreach perm in array permissions
        loop
            execute format('grant %I on %I to %I', perm.actions, perm.tableset, target_user_id);
        end loop;
    end;
$$;

create or replace function _create_person_user_as_user_creator(
    current_user_id uuid, target_user_id uuid
) returns void language plpgsql volatile strict security definer as $$
    begin
        perform _validate_user_can_admin_users(current_user_id);
        perform _validate_user_is_person(target_user_id);

        perform create_app_user_if_not_exists(target_user_id);
    end;
$$;

create or replace function _drop_person_user_as_user_creator(
    current_user_id uuid, target_user_id uuid
) returns void language plpgsql volatile strict security definer as $$
    begin
        perform _validate_user_can_admin_users(current_user_id);
        perform _validate_user_is_person(target_user_id);

        execute format('revoke all on all tables in schema public from %I', target_user_id);
        execute format('drop user %I', target_user_id);
    end;
$$;

create or replace function _revoke_permissions_from_person_as_user_creator(
    current_user_id uuid, service_id uuid, permissions permission_set []
) returns void language plpgsql volatile strict security definer as $$
    declare perm permission_set;
    begin
        perform _validate_user_can_admin_users(current_user_id);
        perform _validate_user_is_person(target_user_id);

        foreach perm in array permissions
        loop
            execute format('revoke %I on %I from %I', perm.actions, perm.tableset, service_id);
        end loop;
    end;
$$;

reset role;


create or replace function grant_permissions_to_person(
    user_id uuid, permissions permission_set []
) returns void language plpgsql volatile strict as $$
    begin
        perform _grant_permissions_to_person_as_user_creator(current_user::uuid, user_id, permissions);
    end;
$$;

create or replace function create_person_user_with_permissions(
    user_id uuid, permissions permission_set []
) returns void language plpgsql volatile strict as $$
    begin
        perform _create_person_user_as_user_creator(current_user::uuid, user_id);
        perform grant_permissions_to_person(user_id, permissions);
    end;
$$;

create or replace function revoke_permissions_from_service(
    service_id uuid, permissions permission_set []
) returns void language plpgsql volatile strict as $$
    begin
        perform _revoke_permissions_from_person_as_user_creator(current_user::uuid, service_id, permissions);
    end;
$$;

create or replace function drop_person_user(user_id uuid) returns void language plpgsql volatile strict as $$
    begin
        perform _drop_person_user_as_user_creator(current_user::uuid, user_id);
    end;
$$;
