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
create policy can_update on principal for update using (
    current_user_can('update', 'person') or current_user_is_service_owner(id)
);
-- Regardless of the above, nobody can make anyone staff unless they are staff themselves
create policy no_privilege_escalation on principal as restrictive for all using (true) with check (
    not is_staff or current_user_is_staff()
);

alter table permission enable row level security;

create policy can_read on permission for select using (true);

create policy can_write on permission for all using (
    -- You can only hand out a permission you own
    current_user_can(action, resource)
    and (
        -- You must either have the "update person" privilege or be the owner of a service
        current_user_can('update', 'person')
        or current_user_is_service_owner(principal_id)
    )
);
