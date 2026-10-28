create function current_user_has_access_to_bot_account(bot_account uuid) returns boolean language sql stable as $$
    select exists (
        select 1 from bot_account_access where bot_account_id = bot_account and person_id = app_user_id()
    )
$$;

alter table bot_account enable row level security;

create policy can_read on bot_account for select using (
    id = app_user_id() or owned_by = app_user_id() or current_user_has_access_to_bot_account(id)
);
create policy can_create on bot_account for insert with check (owned_by = app_user_id());
create policy can_update on bot_account for update using (owned_by = app_user_id());
create policy can_delete on bot_account for delete using (owned_by = app_user_id());

alter table bot_account_access enable row level security;

create policy can_read on bot_account_access for select using (true);
create policy can_create on bot_account_access for insert with check (
    current_user_is_bot_account_owner(bot_account_id)
);
create policy can_delete on bot_account_access for delete using (current_user_is_bot_account_owner(bot_account_id));
