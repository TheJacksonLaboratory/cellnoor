alter table api_key enable row level security;

create policy owners_can_write_api_keys on api_key for all using (
    -- You can modify your own API key, and the owner of a service can modify that service's API keys
    owner_id = app_user_id() or current_user_is_service_owner(owner_id)
);
-- Policies are combined with `or`, so people with access to a service can also see its API keys
create policy service_members_can_see_api_keys on api_key for select using (
    current_user_has_access_to_service(owner_id)
);
