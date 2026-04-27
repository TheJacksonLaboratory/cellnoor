create type project_links as (
    self text,
    specimens text,
    chromium_datasets text
);

-- `security_invoker = true` means that the query's security checks will run as the "invoker" (`current_user`), not the
-- owner of the view
create view project_compact with (security_invoker = true) as (
    select
        *,
        (
            '/projects/' || id, '/projects' || id || '/specimens', '/projects/' || id || '/chromium-datasets'
        )::project_links as links
    from project
);

create view project_to_people as (
    select
        proj as project,
        array(
            select proj_acc.person_id
            from project_access as proj_acc
            where proj_acc.project_id = proj.id
        ) as people
    from project_compact as proj
);
