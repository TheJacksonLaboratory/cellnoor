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

insert into person (id, name, email, institution_id, is_staff, orcid)
select
    uuid_nil(),
    admin_person ->> 'name',
    admin_person ->> 'email',
    uuid_nil(),
    (admin_person ->> 'is_staff')::boolean,
    admin_person ->> 'orcid'
from initial_data;
-- noqa: enable=AL03

-- Create a db user for admin user
select create_person_user_if_not_exists(uuid_nil()::text);

-- Grant them permissions on everything
do $$
    begin
        execute format('grant all on all tables in schema public to %I with grant option', uuid_nil()::text);
        execute format('alter user %I with createrole', uuid_nil()::text);
    end;
$$;

-- 'auth' only needs to create and read people
grant insert, select, update on person, person_account to auth;
grant select on institution to auth;
alter user auth with createrole;

grant select on institution,
person_public,
service_account,
service_account_access,
api_key,
project,
project_access,
project_detailed,
specimen,
specimen_measurement,
specimen_detailed,
tenx_assay,
index_kit,
single_index_set,
dual_index_set,
library_type_specification,
suspension,
suspension_detailed,
suspension_pool_to_specimen,
gem_well_to_specimen,
chromium_cdna_to_specimen,
cdna_preparer,
cdna_measurement,
chromium_library_to_specimen,
library_preparer,
library_measurement,
chromium_dataset_to_specimen,
chromium_dataset_raw_file,
chromium_dataset_parsed_file to public;

grant insert, update, delete on service_account, service_account_access, api_key to public;

alter table service_account enable row level security;
create policy service_account_access on service_account using (
    service_account.id in (
        select service_account_access.service_account_id from service_account_access
        where service_account_access.person_id = current_user::uuid
    )
);

create or replace function current_user_has_access_to_project(
    project_id_to_check uuid
) returns boolean language plpgsql volatile strict as $$
    declare
        has_access boolean;
        current_user_id uuid = current_user::uuid;
    begin
        select is_staff from person_public where id = current_user_id into has_access;
        if has_access then
            return has_access;
        end if;

        select exists (select 1 from project_access where current_user_id in (project_access.person_id, project_access.api_key_id) and project_access.project_id = project_id_to_check) into has_access;
        return has_access;
    end;
$$;

-- Enable row-level security. Note that because every view comes back to specimen, and specimen has
-- `security_invoker = true`, we don't need to enable it for any experimental entities besides projects and specimens
alter table project enable row level security;
create policy project_access on project using (current_user_has_access_to_project(project.id));

alter table specimen enable row level security;
create policy specimen_access on specimen using (current_user_has_access_to_project(specimen.project_id));
