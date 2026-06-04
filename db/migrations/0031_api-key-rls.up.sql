create or replace function current_user_has_access_to_service_account(
    service_account_id_to_check uuid
) returns boolean language plpgsql volatile strict as $$
    declare
        has_access boolean;
        current_user_id uuid = current_user::uuid;
    begin
        select exists (select 1 from service_account_access where current_user_id = service_account_access.person_id and service_account_access.service_account_id = service_account_id_to_check) into has_access;

        return has_access;
    end;
$$;

alter table api_key enable row level security;

create policy api_key_access on api_key using (
    current_user::uuid = person_id
    or current_user_is_service_account_owner(api_key.service_account_id)
    or current_user_has_access_to_service_account(api_key.service_account_id)
);
