-- We need an admin user who can populate the database, so we read this initial data from a JSON file
-- noqa: disable=AL03
create temp table initial_data as select jsonb(pg_read_file('/initial-data.json')) as data;

insert into institution (id, name, microsoft_entra_tenant_id)
select
    uuid_nil(),
    data -> 'admin_institution' ->> 'name',
    (data -> 'admin_institution' ->> 'microsoft_entra_tenant_id')::uuid
from initial_data;

-- Inserting the person also inserts their principal, which the next statement makes staff
insert into person (id, name, institution_id, orcid)
select
    uuid_nil(),
    data -> 'admin' ->> 'name',
    uuid_nil(),
    data -> 'admin' ->> 'orcid'
from initial_data;

insert into account (person_id, auth_provider, auth_provider_user_id)
select
    uuid_nil(),
    data -> 'admin' ->> 'auth_provider',
    data -> 'admin' ->> 'auth_provider_user_id'
from initial_data;

drop table initial_data;

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
-- noqa: enable=AL03
