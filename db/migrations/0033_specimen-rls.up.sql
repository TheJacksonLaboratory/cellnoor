-- Because every view comes back to specimen, and specimen_detailed has
-- `security_invoker = true`, we don't need to enable RLS for any other views
alter table specimen enable row level security;

create policy only_project_members_can_see_specimen on specimen using (
    current_user_is_staff_or_project_creator(specimen.project_id)
    or current_user_has_access_to_project(specimen.project_id)
);
