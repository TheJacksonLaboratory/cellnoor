-- Although this schema tracks lots of entities, they are all intermediate and ephemeral besides specimens, libraries,
-- and Chromium datasets. As such, consumers of the REST API (us and others) will want to filter not only on the fields
-- of a given entity, but also on the fields of its parent specimen (or library, or Chromium dataset), so we create a system of views
-- that ultimately allow us to easily filter on the fields of the starting specimen(s) from any node in the tree. See
-- `0018_suspension-views.up.sql` for an example of the general form of these views. To be precise, the narrow function
-- of these views is to collect the necessary data for filtering, whereas the eventual query decides what to include and
-- how to shape it.

create view person_public as (
    select
        id,
        name,
        institution_id,
        orcid
    from person
);

create view person_to_institution as (
    select
        pers as person,
        org as institution
    from person_public as pers join institution as org on pers.institution_id = org.id
);
