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

insert into person (id, name, email, institution_id, orcid)
select
    uuid_nil(),
    admin_person ->> 'name',
    admin_person ->> 'email',
    uuid_nil(),
    admin_person ->> 'orcid'
from initial_data;
-- noqa: enable=AL03

-- Create a db user for admin user
select create_person_user_if_not_exists(uuid_nil()::text, true);

-- Grant them permissions on everything
do $$
    begin
        execute format('grant all on all tables in schema public to %I', uuid_nil()::text);
        execute format('alter user %I with createrole', uuid_nil()::text);
    end;
$$;

-- 'auth' only needs to create and read people
grant insert, select, update on person, person_account to auth;
grant select on institution to auth;

-- It also needs to create db users
alter user auth with createrole;

-- We also enable row-level security here. Note that because every view comes back to specimen, and specimen has `security_invoker = true`, we don't need to enable it for anything more than these two tables
alter table project enable row level security;

create policy project_access on project using (
    id in (
        select project_id from project_access
        where person_id = current_user::uuid
    )
);

alter table specimen enable row level security;

create policy specimen_access on specimen using (
    project_id in (
        select project_id from project_access
        where person_id = current_user::uuid
    )
);
