-- We need an admin user who can populate the database, so we read this initial data from a JSON file
with initial_data as (
    select json(pg_read_file('/initial-data.json'))->'admin_organization' as admin_organization
)
insert into organization (id, name, microsoft_entra_tenant_id) select uuid_nil(), admin_organization->>'name', (admin_organization->>'microsoft_entra_tenant_id')::uuid from initial_data;

with initial_data as (
    select json(pg_read_file('/initial-data.json'))->'admin' as admin_person
)
insert into person (id, name, email, organization_id, orcid) select uuid_nil(), admin_person->>'name', admin_person->>'email', uuid_nil(), admin_person->>'orcid' from initial_data;

-- We also need to grant permissions to the admin user
select create_person_user_if_not_exists(uuid_nil()::text, true);
do $$
    begin
        execute format('grant all on all tables in schema public to %I', uuid_nil()::text);
        execute format('alter user %I with createrole', uuid_nil()::text);
    end;
$$;

-- 'auth' only needs to create and read people
grant insert, select, update on person, person_account to auth;
grant select on organization to auth;

-- It also needs to create db users
alter user auth with createrole;
