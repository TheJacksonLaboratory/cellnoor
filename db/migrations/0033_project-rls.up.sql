create or replace function current_user_has_access_to_project(
    project_id_to_check uuid
) returns boolean language plpgsql volatile strict as $$
    declare
        has_access boolean;
        current_user_id uuid = current_user::uuid;
    begin
        select exists (select 1 from project_access where current_user_id in (project_access.person_id, project_access.service_id) and project_access.project_id = project_id_to_check) into has_access;

        return has_access;
    end;
$$;

alter table project enable row level security;

create or replace function current_service_is_staff() returns boolean language plpgsql volatile strict as $$
    declare
        current_user_id uuid = current_user::uuid;
        can_read boolean;
    begin
        select is_staff from service where id = current_user_id into can_read;
        return can_read;
    end;
$$;

create or replace function current_user_is_staff() returns boolean language plpgsql volatile strict as $$
    begin
        return current_person_is_staff() or current_service_is_staff();
    end;
$$;

create policy select_project on project for select using (
    current_user::uuid in (created_by_person, created_by_service)
    or current_user_is_staff()
    or current_user_has_access_to_project(project.id)
);

create policy insert_project on project for insert with check (
    current_user::uuid in (project.created_by_person, project.created_by_service)
    or current_user_has_access_to_project(project.id)
);

-- Note that a user still has to be explicitly granted the update and delete privileges, but they also need to be
-- "staff-like", which is to say they can see all projects
create policy update_project on project for update using (current_user_is_staff());
create policy delete_project on project for delete using (current_user_is_staff());

-- Again, we can't use this function for the above RLS policies because it'd cause infinite recursion
create or replace function current_user_is_project_creator(
    project_id_to_check uuid
) returns boolean language plpgsql volatile strict as $$
    declare
        is_project_creator boolean;
        current_user_id uuid = current_user::uuid;
    begin
        select current_user_id in (created_by_person, created_by_service) from project where id = project_id_to_check into is_project_creator;

        return is_project_creator;
    end;
$$;

create or replace function current_user_is_staff_or_is_project_creator(
    project_id_to_check uuid
) returns boolean language plpgsql volatile strict as $$
    begin
        return current_user_is_staff() or current_user_is_project_creator(project_id_to_check);
    end;
$$;

alter table project_access enable row level security;

create policy anyone_can_see_project_membership on project_access for select using (true);
create policy admin_and_creator_can_add_others on project_access for insert with check (
    current_user_is_staff_or_is_project_creator(project_access.project_id)
);
create policy admin_and_creator_can_remove_others on project_access for delete using (
    current_user_is_staff_or_is_project_creator(project_access.project_id)
);
