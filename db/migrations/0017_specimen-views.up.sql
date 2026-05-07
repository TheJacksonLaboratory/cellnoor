-- I hate sqlfluff
-- noqa: disable=AL05
-- We have to enable `security_invoker` here so that queries against this view use row-level security
create view specimen_detailed with (security_invoker = true) as (
    select
        specimen,
        project,
        array(
            select specimen_measurement from specimen_measurement
            where specimen_id = specimen.id
        ) as measurements
    from specimen join project on specimen.project_id = project.id
);
-- noqa: enable=AL05
