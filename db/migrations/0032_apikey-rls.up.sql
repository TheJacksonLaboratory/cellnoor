alter table api_key enable row level security;

create policy api_key_access on api_key using (
    current_user::uuid = person_id
    or current_user_is_service_owner(api_key.service_id)
);

-- This policy is combined using `or` with the above policy, so we don't need to repeat everything above
create policy select_api_key on api_key for select using (current_user_has_access_to_service(api_key.service_id));
