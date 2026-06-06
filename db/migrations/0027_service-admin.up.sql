create or replace function user_is_service(user_id uuid) returns boolean language plpgsql volatile strict as $$
    declare is_service boolean;
    begin
        select exists (select 1 from service where id = user_id) into is_service;
        return is_service;
    end;
$$;

set role user_creator;

create or replace function _user_has_permission_set_on_tableset(
    user_id uuid, permission permission_set
) returns boolean language plpgsql volatile strict security definer as $$
    declare
        has_permissions boolean = false;
        table_ text;
        action_ text;
    begin
        foreach table_ in array string_to_array(perm.tableset, ',')
        loop
            foreach action_ in array string_to_array(perm.actions, ',')
            loop
                has_permissions = (has_permissions and has_table_privilege(current_user_id, table_, action_));
            end loop;
        end loop;

        return has_permissions;
    end;
$$;

create or replace function _validate_user_is_service_owner(
    current_user_id uuid, service_id uuid
) returns void language plpgsql volatile strict security definer as $$
    declare is_service_owner boolean;
    begin
        select current_user_id = owned_by from service where id = service_id into is_service_owner;
        if not is_service_owner then
            raise insufficient_privilege using message = 'must be service owner';
        end if;
    end;
$$;

create or replace function _grant_permissions_to_service_as_user_creator(
    current_user_id uuid, service_id uuid, permissions permission_set []
) returns void language plpgsql volatile strict security definer as $$
    declare
        user_is_service_owner boolean;
        perm permission_set;
    begin
        perform _validate_user_is_service_owner(current_user_id, service_id);

        foreach perm in array permissions
        loop
            if not _user_has_permission_set_on_tableset(current_user_id, perm) then
                raise insufficient_privilege using message = 'user cannot grant this permission to service';
            end if;
            execute format('grant %I on %I to %I', perm.actions, perm.tableset, service_id);
        end loop;
    end;
$$;

create or replace function _create_service_user_as_user_creator(
    current_user_id uuid, service_id uuid
) returns void language plpgsql volatile strict security definer as $$
    begin
        perform _validate_user_is_service_owner(current_user_id, service_id);

        perform create_app_user_if_not_exists(service_id);
        execute format('alter user %I nocreaterole', service_id);
    end;
$$;

create or replace function _revoke_permissions_from_service_as_user_creator(
    current_user_id uuid, service_id uuid, permissions permission_set []
) returns void language plpgsql volatile strict security definer as $$
    declare perm permission_set;
    begin
        perform _validate_user_is_service_owner(current_user_id, service_id);

        foreach perm in array permissions
        loop
            execute format('revoke %I on %I from %I', perm.actions, perm.tableset, service_id);
        end loop;
    end;
$$;

create or replace function _drop_service_user_as_user_creator(
    current_user_id uuid, service_id uuid
) returns void language plpgsql volatile strict security definer as $$
    declare user_is_service_owner boolean;
    begin
        perform _validate_user_is_service_owner(current_user_id, service_id);

        execute format('revoke all on all tables in schema public from %I', service_id);
        execute format('drop user %I', service_id);
    end;
$$;

reset role;

create or replace function grant_permissions_to_service(
    service_id uuid, permissions permission_set []
) returns void language plpgsql volatile strict as $$
    begin
        perform _grant_permissions_to_service_as_user_creator(current_user::uuid, service_id, permissions);
    end;
$$;

create or replace function create_service_user_with_permissions(
    service_id uuid, permissions permission_set []
) returns void language plpgsql volatile strict as $$
    begin
        perform _create_service_user_as_user_creator(current_user::uuid, service_id);
        perform grant_permissions_to_service(current_user::uuid, service_id, permissions);
    end;
$$;

create or replace function revoke_permissions_from_service(
    service_id uuid, permissions permission_set []
) returns void language plpgsql volatile strict as $$
    begin
        perform _revoke_permissions_from_service_as_user_creator(current_user::uuid, service_id, permissions);
    end;
$$;

create or replace function drop_service_user(service_id uuid) returns void language plpgsql volatile strict as $$
    begin
        perform _drop_service_user_as_user_creator(current_user::uuid, service_id);
    end;
$$;
