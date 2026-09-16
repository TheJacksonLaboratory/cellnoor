-- We need an admin user who can populate the database, so we read this initial data from a JSON file
-- noqa: disable=AL03
with initial_data as (
    select json(pg_read_file('/initial-data.json')) -> 'admin_institution' as admin_institution
)

insert into institution (id, name, microsoft_entra_tenant_id)
select
    uuid_nil(),
    admin_institution ->> 'name',
    (admin_institution ->> 'microsoft_entra_tenant_id')::uuid
from initial_data;


with initial_data as (
    select json(pg_read_file('/initial-data.json')) -> 'admin' as admin_person
)

insert into person (id, name, institution_id, orcid)
select
    uuid_nil(),
    admin_person ->> 'name',
    uuid_nil(),
    admin_person ->> 'orcid'
from initial_data;

-- The admin sees every project and may do anything to every resource
update principal set is_staff = true
where id = uuid_nil();

insert into permission (principal_id, resource, action)
select
    uuid_nil(),
    resource,
    action
from
    unnest(array[
        'institution',
        'person',
        'account',
        'project',
        'specimen',
        'assay_constant_data',
        'chromium_experimental_data',
        'chromium_dataset'
    ]) as resource
cross join unnest(array['create', 'update', 'delete']) as action;

with initial_data as (
    select json(pg_read_file('/initial-data.json')) -> 'admin' as admin_person
)

insert into account (
    person_id, auth_provider, auth_provider_user_id
)
select
    uuid_nil(),
    admin_person ->> 'auth_provider',
    admin_person ->> 'auth_provider_user_id'
from initial_data;
-- noqa: enable=AL03
