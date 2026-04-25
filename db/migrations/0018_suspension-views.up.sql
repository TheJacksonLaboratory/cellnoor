create view suspension_brief as (
    select
        *,
        row('/suspensions/' || id)::simple_links as links
    from suspension
);

create view suspension_full as (
    select
        susp as suspension,
        spec as specimen,
        array(
            select mes from suspension_measurement as mes
            where mes.suspension_id = susp.id
        ) as measurements,
        array(
            select prep.prepared_by
            from suspension_preparer as prep
            where prep.suspension_id = susp.id
        ) as preparers
    from suspension_brief as susp join specimen_brief as spec on susp.specimen_id = spec.id
);
