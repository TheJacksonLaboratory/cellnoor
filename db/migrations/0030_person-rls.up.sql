alter table person enable row level security;

-- Note that this means anyone can delete anyone, but this is a privilege we have to grant explicitly, so we're safe
create policy anyone_can_view_anyone on person using (true);

create or replace function current_person_can_admin_all_projects() returns boolean language plpgsql volatile strict as $$
    declare
        current_user_id uuid = current_user::uuid;
        can_admin boolean;
    begin
        select can_admin_all_projects from person where id = current_user_id into can_admin;
        return can_admin;
    end;
$$;

-- If the person we are creating isn't getting can_admin_users, then proceed. If they are, then the
-- current_user also has to have can_admin_users. Same with `can_view_all_projects`
create policy only_grant_owned_perms on person with check (
    ((not can_admin_users) or user_can_admin_users(current_user::uuid))
    and ((not can_admin_all_projects) or current_person_can_admin_all_projects())
);
