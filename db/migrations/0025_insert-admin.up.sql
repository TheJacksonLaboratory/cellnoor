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

insert into person (
    id, name, institution_id, is_staff, can_manage_users, orcid
)
select
    uuid_nil(),
    admin_person ->> 'name',
    uuid_nil(),
    (admin_person ->> 'is_staff')::boolean,
    true,
    admin_person ->> 'orcid'
from initial_data;

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

-- Create a db user for admin user
select create_app_user_if_not_exists(uuid_nil());

-- Grant them permissions on everything
do $$
    begin
        execute format('grant all on all tables in schema public to %I with grant option', uuid_nil());
        execute format('grant %I to auth with admin true, inherit false', uuid_nil());
    end;
$$;
