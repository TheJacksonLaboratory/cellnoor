alter table api_key enable row level security;

create policy api_key_access on api_key using (
    current_user::uuid = person_id
    or current_user_is_service_account_owner(api_key.service_account_id)
    or current_user_has_access_to_service_account(api_key.service_account_id)
);
