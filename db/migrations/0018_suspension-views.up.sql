-- Here, we expose a couple of different things in the view:
--   1. The leaf we're looking at (suspension)
--   2. The parent specimen from which this leaf derives
-- We don't expose a suspension's measurements or preparers because queries like /suspensions?view=compact will not
-- return those, so there's no reason to have them in this view.
create view suspension_to_specimen with (security_invoker = true) as (
    select
        suspension,
        specimen
    from suspension join specimen on suspension.specimen_id = specimen.id
);
