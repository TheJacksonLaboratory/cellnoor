create function current_user_has_access_to_service(service uuid) returns boolean language sql stable as $$
    select exists (
        select 1 from service_access where service_id = service and person_id = app_user_id()
    )
$$;

alter table service enable row level security;

create policy owners_and_members_can_see_services on service for select using (
    id = app_user_id() or owned_by = app_user_id() or current_user_has_access_to_service(id)
);
create policy owners_can_write_services on service for all using (id = app_user_id() or owned_by = app_user_id());

alter table service_access enable row level security;

create policy anyone_can_see_service_access on service_access for select using (true);
create policy owners_can_write_service_access on service_access for all using (
    -- You can give access to a service if you have access to it
    current_user_is_service_owner(service_id) or current_user_has_access_to_service(service_id)
);
