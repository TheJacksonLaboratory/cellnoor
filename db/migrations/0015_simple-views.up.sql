-- There are 3 things to note about this system of views:
-- 1. Every entity has a "brief" view and a "full" view. The former is for API requests like "/organizations", and the
-- latter is for API requests like "/organizations/{id}"
--
-- 2. The `_full` view should include relevant children entities, unless there are too many. For example, a
-- `chromium_dataset_full` should have an array of its libraries and its suspensions (complete with tagging) because
-- there aren't many libraries or suspensions/suspension pools in a Chromium dataset. However, a project may have many
-- Chromium datasets, so we stick that in a separate link.
--
-- 3. Generally, any aggregations of "people" (like a suspension's preparers) should just collect the people's IDs.
-- It's not necessary to join all the way up to a `person_brief`.
create view organization_full as (
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

create view person_brief as (
    select
        id,
        name,
        organization_id,
        orcid,
        ('/people/' || id, '/people/' || id || '/projects', '/people/' || id || '/specimens')::person_links as links
    from person
);

create view person_full as (
    select
        pers as person,
        org as organization
    from person_brief as pers join organization_full as org on pers.organization_id = org.id
);
