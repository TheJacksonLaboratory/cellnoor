create view project_detailed with (security_invoker = true) as (
    select
        project,
        array(
            select principal_id from project_access
            where project_access.project_id = project.id
        ) as members
    from project
);
