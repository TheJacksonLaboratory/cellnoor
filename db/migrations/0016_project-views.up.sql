

-- `security_invoker = true` means that the query's security checks will run as the "invoker" (`current_user`), not the
-- owner of the view
create view project_to_people with (security_invoker = true) as (
    select
        proj as project,
        array(
            select proj_acc.person_id
            from project_access as proj_acc
            where proj_acc.project_id = proj.id
        ) as people
    from project as proj
);
