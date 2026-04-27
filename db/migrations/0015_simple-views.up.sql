-- Although this schema tracks lots of entities, they are all intermediate and ephemeral besides specimens, libraries,
-- and Chromium datasets. As such, consumers of the REST API (us) will want to filter not only on the fields of a given
-- leaf, but also on the fields of its parent specimen (or library, or Chromium dataset), so we create a system of views
-- that ultimately allow us to easily filter on the fields of the starting specimen(s) from any node in the tree. See
-- `0018_suspension-views.up.sql` for an example of the general form of these views. To be precise, the narrow function
-- of these views is to collect the necessary data for filtering, whereas the eventual query decides what to include and
-- how to shape it.

create view organization_compact as (
    select
        *,
        row('/organizations/' || id)::simple_links as links
    from organization
);

create type person_links as (
    self text,
    projects text,
    specimens text
);

create view person_compact as (
    select
        id,
        name,
        organization_id,
        orcid,
        ('/people/' || id, '/people/' || id || '/projects', '/people/' || id || '/specimens')::person_links as links
    from person
);

create view person_to_organization as (
    select
        pers as person,
        org as organization
    from person_compact as pers join organization_compact as org on pers.organization_id = org.id
);
