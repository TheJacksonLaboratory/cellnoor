create function current_user_has_access_to_service(service uuid) returns boolean language sql stable as $$
    select exists (
        select 1 from service_access where service_id = service and person_id = app_user_id()
    )
$$;

alter table service enable row level security;

create policy can_read on service for select using (
    id = app_user_id() or owned_by = app_user_id() or current_user_has_access_to_service(id)
);
create policy can_create on service for insert with check (owned_by = app_user_id());
create policy can_update on service for update using (owned_by = app_user_id());
create policy can_delete on service for delete using (owned_by = app_user_id());

alter table service_access enable row level security;

create policy can_read on service_access for select using (true);
create policy can_create on service_access for insert with check (current_user_is_service_owner(service_id));
create policy can_delete on service_access for delete using (current_user_is_service_owner(service_id));
