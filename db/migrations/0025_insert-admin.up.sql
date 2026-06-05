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

insert into person (id, name, email, email_verified, institution_id, is_staff, orcid)
select
    uuid_nil(),
    admin_person ->> 'name',
    admin_person ->> 'email',
    true,
    uuid_nil(),
    (admin_person ->> 'is_staff')::boolean,
    admin_person ->> 'orcid'
from initial_data;
-- noqa: enable=AL03

-- Create a db user for admin user
select create_person_user_if_not_exists(uuid_nil());

-- Grant them permissions on everything
do $$
    begin
        execute format('grant all on all tables in schema public to %I with grant option', uuid_nil());
        execute format('alter user %I with createrole', uuid_nil());
    end;
$$;
