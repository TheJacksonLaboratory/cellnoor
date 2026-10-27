-- These functions run as the application user, so row-level security applies to the tables they read. That's fine
-- because everyone may read `principal` and `permission`, and a service is visible to its owner
create function current_user_is_staff() returns boolean language sql stable as $$
    select coalesce((select is_staff from principal where id = app_user_id()), false)
$$;

create function current_user_can(action_ text, resource_ text) returns boolean language sql stable as $$
    select exists (
        select 1 from permission
        where principal_id = app_user_id() and resource = resource_ and action = action_
    )
$$;

create function current_user_is_service_owner(
    service_id_to_check uuid
) returns boolean language sql stable as $$
    select exists (select 1 from service where id = service_id_to_check and owned_by = app_user_id())
$$;

alter table principal enable row level security;

create policy can_read on principal for select using (true);
-- Nobody can make anyone staff unless they are staff themselves
create policy can_update on principal for update using (
    (current_user_is_staff() and current_user_can('update', 'person')) or current_user_is_service_owner(id)
) with check (not is_staff or current_user_is_staff());

alter table permission enable row level security;

create policy can_read on permission for select using (true);
-- You can only grant or revoke a permission you hold, and only for a person (as staff who may update people) or for a
-- service you own
create policy can_create on permission for insert with check (
    current_user_can(action, resource)
    and (
        (
            current_user_is_staff()
            and current_user_can('update', 'person')
            and exists (
                select 1 from person
                where id = principal_id
            )
        )
        or current_user_is_service_owner(principal_id)
    )
);
create policy can_delete on permission for delete using (
    current_user_can(action, resource)
    and (
        (
            current_user_is_staff()
            and current_user_can('update', 'person')
            and exists (
                select 1 from person
                where id = principal_id
            )
        )
        or current_user_is_service_owner(principal_id)
    )
);
