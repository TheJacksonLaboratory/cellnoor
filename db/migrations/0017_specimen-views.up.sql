-- I hate sqlfluff
-- noqa: disable=AL05
-- We have to enable `security_invoker` here so that queries against this view use row-level security

create view specimen_to_project with (security_invoker = true) as (
    select
        specimen,
        project
    from specimen join project on specimen.project_id = project.id
);

create view specimen_detailed with (security_invoker = true) as (
    select
        specimen,
        project,
        array(
            select specimen_measurement from specimen_measurement
            where specimen_id = (specimen_to_project.specimen).id
        ) as measurements
    from specimen_to_project
);
-- noqa: enable=AL05
