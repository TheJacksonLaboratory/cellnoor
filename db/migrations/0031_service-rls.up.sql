create or replace function current_user_has_access_to_service(
    service_id_to_check uuid
) returns boolean language plpgsql volatile strict as $$
    declare
        has_access boolean;
        current_user_id uuid = current_user::uuid;
    begin
        select exists (select 1 from service_access where current_user_id = service_access.person_id and service_access.service_id = service_id_to_check) into has_access;

        return has_access;
    end;
$$;

alter table service enable row level security;

create policy select_service on service for select using (
    id = current_user::uuid
    or owned_by = current_user::uuid
    or current_user_has_access_to_service(service.id)
);

create policy write_service on service using (
    id = current_user::uuid or owned_by = current_user::uuid
) with check (
    (id = current_user::uuid or owned_by = current_user::uuid)
    and ((not can_manage_users) or user_can_manage_users(current_user::uuid))
    and ((not is_staff) or current_person_is_staff())
);

-- This function seems like it'd be useful in the above RLS policies, but it would cause infinite recursion
create or replace function current_user_is_service_owner(
    service_id_to_check uuid
) returns boolean language plpgsql volatile strict as $$
    declare
        owner uuid;
        current_user_id uuid = current_user::uuid;
    begin
        select owned_by from service where id = service_id_to_check into owner;

        return owner = current_user_id;
    end;
$$;

alter table service_access enable row level security;

create policy anyone_can_see_service_access on service_access for select using (true);
create policy owner_can_add_others on service_access using (
    current_user_is_service_owner(service_id)
);
