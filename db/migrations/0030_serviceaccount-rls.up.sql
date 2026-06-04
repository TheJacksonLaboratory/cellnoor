alter table service_account enable row level security;

create policy owner_can_insert_service_account on service_account for insert with check (owned_by = current_user::uuid);
create policy owner_can_update_service_account on service_account for update using (owned_by = current_user::uuid);
create policy select_service_account on service_account for select using (
    owned_by = current_user::uuid or current_user_has_access_to_service_account(service_account.id)
);
create policy delete_service_account on service_account for delete using (owned_by = current_user::uuid);

-- This function seems like it'd be useful in the above RLS policies, but it would cause infinite recursion
create or replace function current_user_is_service_account_owner(
    service_account_id_to_check uuid
) returns boolean language plpgsql volatile strict as $$
    declare
        account_owner uuid;
        current_user_id uuid = current_user::uuid;
    begin
        select owned_by from service_account where id = service_account_id_to_check into account_owner;

        return account_owner = current_user_id;
    end;
$$;

alter table service_account_access enable row level security;

create policy anyone_can_see_service_account_access on service_account_access for select using (true);
create policy owner_can_add_others on service_account_access for insert with check (
    current_user_is_service_account_owner(service_account_id)
);
create policy owner_can_remove_others on service_account_access for delete using (
    current_user_is_service_account_owner(service_account_id)
);
