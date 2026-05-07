-- Here, we expose a couple of different things in the view:
--   1. The leaf we're looking at (suspension)
--   2. The parent specimen from which this leaf derives
create view suspension_to_specimen with (security_invoker = true) as (
    select
        suspension,
        specimen
    from suspension join specimen on suspension.specimen_id = specimen.id
);

create view suspension_detailed as (
    select
        suspension,
        specimen,
        array(
            select mes from suspension_measurement as mes
            where mes.suspension_id = (suspension).id
        ) as measurements,
        array(
            select prep.prepared_by from suspension_preparer as prep
            where prep.suspension_id = (suspension).id
        ) as preparers
    from suspension_to_specimen
);
