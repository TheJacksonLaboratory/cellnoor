alter table api_key enable row level security;

create policy api_key_access on api_key for select using (
    current_user::uuid = person_id
    or current_user_is_service_owner(api_key.service_id)
    or current_user_has_access_to_service(api_key.service_id)
);

create policy write_api_key on api_key with check (
    current_user::uuid = person_id
    or current_user_is_service_owner(api_key.service_id)
);

create policy delete_api_key on api_key for delete using (
    current_user::uuid = person_id
    or current_user_is_service_owner(api_key.service_id)
);
