alter table api_key enable row level security;

-- You can modify your own API key, and the owner of a bot_account can modify that bot_account's API keys. People with
-- access to a bot_account can see its API keys
create policy can_read on api_key for select using (
    owner_id = app_user_id()
    or current_user_is_bot_account_owner(owner_id)
    or current_user_has_access_to_bot_account(owner_id)
);
create policy can_create on api_key for insert with check (
    owner_id = app_user_id() or current_user_is_bot_account_owner(owner_id)
);
create policy can_update on api_key for update using (
    owner_id = app_user_id() or current_user_is_bot_account_owner(owner_id)
);
create policy can_delete on api_key for delete using (
    owner_id = app_user_id() or current_user_is_bot_account_owner(owner_id)
);
