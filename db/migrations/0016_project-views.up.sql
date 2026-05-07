-- `security_invoker = true` means that the query's security checks will run as the "invoker" (`current_user`), not the
-- owner of the view
create view project_detailed with (security_invoker = true) as (
    select
        project,
        array(
            select proj_acc.person_id
            from project_access as proj_acc
            where proj_acc.project_id = project.id
        ) as people
    from project
);
