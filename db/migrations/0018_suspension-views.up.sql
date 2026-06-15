-- Although this schema tracks lots of entities, they are all intermediate and ephemeral besides specimens, libraries,
-- and Chromium datasets. As such, consumers of the REST API (us and others) will want to filter not only on the fields
-- of a given entity, but also on the fields of its parent specimen (or library, or Chromium dataset), so we create a
-- system of views that ultimately allow us to easily filter on the fields of the starting specimen(s) from any node
-- in the tree. To be precise, the narrow function of these views is to collect the necessary data for filtering,
-- whereas the eventual query decides what to include and how to shape it.

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
