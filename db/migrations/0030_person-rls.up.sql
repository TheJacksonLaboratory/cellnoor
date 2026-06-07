alter table person enable row level security;

-- Note that this means anyone can delete anyone, but this is a privilege we have to grant explicitly, so we're safe
create policy anyone_can_read_anyone on person using (true);

create or replace function current_person_is_staff() returns boolean language plpgsql volatile strict as $$
    declare
        current_user_id uuid = current_user::uuid;
        can_read boolean;
    begin
        select is_staff from person_public where id = current_user_id into can_read;
        return can_read;
    end;
$$;

-- If the person we are creating isn't getting can_admin_users, then proceed. If they are, then the
-- current_user also has to have can_admin_users. Same with `is_staff`
create policy can_only_grant_owned_perms on person with check (
    ((not can_admin_users) or user_can_admin_users(current_user::uuid))
    and ((not is_staff) or current_person_is_staff())
);
