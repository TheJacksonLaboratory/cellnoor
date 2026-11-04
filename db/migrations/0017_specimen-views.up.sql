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
        ) as measurements,
        array(
            with recursive ancestor as (
                select
                    parent,
                    1 as depth
                from specimen as parent
                where parent.id = specimen.parent_specimen_id

                union all

                select
                    grandparent,
                    ancestor.depth + 1
                from ancestor join specimen as grandparent on (ancestor.parent).parent_specimen_id = grandparent.id
            )

            select parent from ancestor
            order by depth
        ) as ancestors
    from specimen join project on specimen.project_id = project.id
);
-- noqa: enable=AL05
