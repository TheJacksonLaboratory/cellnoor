create or replace function current_user_has_access_to_project(
    project_id_to_check uuid
) returns boolean language plpgsql volatile strict as $$
    declare
        has_access boolean;
        current_user_id uuid = current_user::uuid;
    begin
        select exists (select 1 from project_access where current_user_id in (project_access.person_id, project_access.api_key_id) and project_access.project_id = project_id_to_check) into has_access;

        return has_access;
    end;
$$;

alter table project enable row level security;

create policy staff_and_members_can_view_project on project for select using (
    current_user_is_staff() or created_by = current_user::uuid or current_user_has_access_to_project(project.id)
);
create policy staff_and_creator_can_create_project on project for insert with check (
    current_user_is_staff() or created_by = current_user::uuid
);
create policy staff_can_update_project on project for update using (current_user_is_staff());
create policy staff_can_delete_project on project for delete using (current_user_is_staff());

-- Again, we can't use this function for the above RLS policies because it'd cause infinite recursion
create or replace function current_user_is_project_creator(
    project_id_to_check uuid
) returns boolean language plpgsql volatile strict as $$
    declare
        project_creator uuid;
        current_user_id uuid = current_user::uuid;
    begin
        select created_by from project where id = project_id_to_check into project_creator;

        return project_creator = current_user_id;
    end;
$$;

create or replace function current_user_is_staff_or_project_creator(
    project_id_to_check uuid
) returns boolean language plpgsql volatile strict as $$
    begin
        return current_user_is_staff() or current_user_is_project_creator(project_id_to_check);
    end;
$$;

alter table project_access enable row level security;

create policy anyone_can_see_project_membership on project_access for select using (true);
create policy staff_and_creator_can_add_others on project_access for insert with check (
    current_user_is_staff_or_project_creator(project_access.project_id)
);
create policy staff_and_creator_can_remove_others on project_access for delete using (
    current_user_is_staff_or_project_creator(project_access.project_id)
);
