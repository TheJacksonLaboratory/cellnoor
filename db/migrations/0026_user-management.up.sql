-- In order for the user `app` to view all the API keys, it needs to bypass RLS and also access them directly
alter user app with bypassrls;
grant select on api_key to app;

-- user_creator needs to create roles and have all permissions so that it can grant permissions. It also needs to bypass
-- RLS
grant all on all tables in schema public to user_creator with grant option;
alter user user_creator createrole bypassrls;
grant create on schema public to user_creator;

create or replace function user_can_manage_users(user_id uuid) returns boolean language plpgsql volatile strict as $$
    declare can_manage boolean;
    begin
        select can_manage_users from person where id = user_id into can_manage;
        return can_manage;
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

create or replace function _validate_user_can_manage_users(
    user_id uuid
) returns void language plpgsql volatile strict as $$
    begin
        if not user_can_manage_users(user_id) then
            raise insufficient_privilege using message = 'permission denied to perform admin actions on users';
        end if;
    end;
$$;

create or replace function _validate_target_user_is_person(
    user_id uuid
) returns void language plpgsql volatile strict as $$
    begin
        if not user_is_person(user_id) then
            raise check_violation using message = 'target user is not a person';
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
        perform _validate_user_can_manage_users(current_user_id);
        perform _validate_target_user_is_person(target_user_id);

        foreach perm in array permissions
        loop
            execute format('grant %s on %s to %I', perm.actions, perm.tableset, target_user_id);
        end loop;
    end;
$$;

create or replace function _create_person_user_as_user_creator(
    current_user_id uuid, target_user_id uuid
) returns void language plpgsql volatile strict security definer as $$
    begin
        perform _validate_user_can_manage_users(current_user_id);
        perform _validate_target_user_is_person(target_user_id);

        perform create_app_user_if_not_exists(target_user_id);
    end;
$$;

create or replace function _drop_person_user_as_user_creator(
    current_user_id uuid, target_user_id uuid
) returns void language plpgsql volatile strict security definer as $$
    begin
        perform _validate_user_can_manage_users(current_user_id);
        perform _validate_target_user_is_person(target_user_id);

        execute format('revoke all on all tables in schema public from %I', target_user_id);
        execute format('drop user %I', target_user_id);
    end;
$$;

create or replace function _revoke_permissions_from_person_as_user_creator(
    current_user_id uuid, target_user_id uuid, permissions permission_set []
) returns void language plpgsql volatile strict security definer as $$
    declare perm permission_set;
    begin
        perform _validate_user_can_manage_users(current_user_id);
        perform _validate_target_user_is_person(target_user_id);

        foreach perm in array permissions
        loop
            execute format('revoke %s on %s from %I', perm.actions, perm.tableset, target_user_id);
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

create or replace function revoke_permissions_from_person(
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
