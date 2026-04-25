create type project_links as (
    self text,
    specimens text,
    chromium_datasets text
);

create view project_brief as (
    select
        *,
        (
            '/projects/' || id, '/projects' || id || '/specimens', '/projects/' || id || '/chromium-datasets'
        )::project_links as links
    from project
);

create view project_full as (
    select
        proj as project,
        array(
            select proj_acc.person_id
            from project_access as proj_acc
            where proj_acc.project_id = proj.id
        ) as people
    from project_brief as proj
);
