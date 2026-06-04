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
grant execute on function create_person_user_if_not_exists to auth;
alter user auth with createrole;

-- 'app' needs to be able to read API keys directly to authenticate people
grant select on api_key to app;

-- Only grant select on the tables and views a user accesses directly to prevent the developer from forgetting that
-- there are convenient views already made
grant select on institution,
person_public,
service_account,
service_account_access,
api_key_public,
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
suspension_pool_measurement,
suspension_pool_preparer,
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

create or replace function current_user_is_staff() returns boolean language plpgsql volatile strict as $$
    declare
        user_is_staff boolean;
        current_user_id uuid = current_user::uuid;
    begin
        -- The simple case: it's just a person
        select is_staff
        from person_public
        where id = current_user_id into user_is_staff;

        if user_is_staff then
            return user_is_staff;
        end if;

        -- The next case: it's a person using an API key
        select pers.is_staff
        from api_key as ak
        join person_public as pers on ak.person_id = pers.id
        where ak.id = current_user_id
        into user_is_staff;

        if user_is_staff then
            return user_is_staff;
        end if;

        -- Finally: it's a service account, which should have `is_staff == all(person.is_staff)` for every person who can use the service account
        select all (pers.is_staff) and svc_owner.is_staff
        from service_account_access as svc_acc
        join service_account as svc on svc_acc.service_account_id = svc.id
        join person_public as pers on svc_acc.person_id = pers.id
        join person_public as svc_owner on svc.owned_by = svc_owner.id
        where svc.id = current_user_id
        into user_is_staff;

        return user_is_staff;
    end;
$$;

create or replace function current_user_has_access_to_service_account(
    service_account_id_to_check uuid
) returns boolean language plpgsql volatile strict as $$
    declare
        has_access boolean;
        current_user_id uuid = current_user::uuid;
    begin
        select exists (select 1 from service_account_access where current_user_id = service_account_access.person_id and service_account_access.service_account_id = service_account_id_to_check) into has_access;

        return has_access;
    end;
$$;

grant insert (description, owned_by), update (description, owned_by), delete on service_account to public;
alter table service_account enable row level security;
create policy owner_can_insert_service_account on service_account for insert with check (owned_by = current_user::uuid);
create policy owner_can_update_service_account on service_account for update using (owned_by = current_user::uuid);
create policy select_service_account on service_account for select using (
    owned_by = current_user::uuid or current_user_has_access_to_service_account(service_account.id)
);
create policy delete_service_account on service_account for delete using (owned_by = current_user::uuid);

-- We can't use this function in the RLS policies for service_account because it would cause infinite recursion
create or replace function current_user_is_service_account_owner(
    service_account_id_to_check uuid
) returns boolean language plpgsql volatile strict as $$
    declare
        account_owner uuid;
        current_user_id uuid = current_user::uuid;
    begin
        select owned_by from service_account where id = service_account_id_to_check into account_owner;

        return account_owner = current_user_id;
    end;
$$;

grant insert, delete on service_account_access to public;
alter table service_account_access enable row level security;
create policy anyone_can_see_service_account_access on service_account_access for select using (true);
create policy owner_can_add_others on service_account_access for insert with check (
    current_user_is_service_account_owner(service_account_id)
);
create policy owner_can_remove_others on service_account_access for delete using (
    current_user_is_service_account_owner(service_account_id)
);

grant insert (description, hashed_key, person_id, service_account_id, expires_at),
select (id, description, person_id, service_account_id, created_at, expires_at),
update (description, expires_at),
delete on api_key to public;
alter table api_key enable row level security;
create policy api_key_access on api_key using (
    current_user::uuid = person_id
    or current_user_is_service_account_owner(api_key.service_account_id)
    or current_user_has_access_to_service_account(api_key.service_account_id)
);

create or replace function current_user_has_access_to_project(
    project_id_to_check uuid
) returns boolean language plpgsql volatile strict as $$
    declare
        has_access boolean;
        current_user_id uuid = current_user::uuid;
    begin
        select exists (select 1 from project_access where current_user_id in (project_access.person_id, project_access.api_key_id) and project_access.project_id = project_id_to_check) into has_access;

        return has_access;
    end;
$$;

alter table project enable row level security;
create policy staff_and_members_can_view_project on project for select using (
    current_user_is_staff() or created_by = current_user::uuid or current_user_has_access_to_project(project.id)
);
create policy staff_and_creator_can_create_project on project for insert with check (
    current_user_is_staff() or created_by = current_user::uuid
);
create policy staff_can_update_project on project for update using (current_user_is_staff());
create policy staff_can_delete_project on project for delete using (current_user_is_staff());

-- Again, we can't use this function for RLS policies on project because it'd cause infinite recursion
create or replace function current_user_is_project_creator(
    project_id_to_check uuid
) returns boolean language plpgsql volatile strict as $$
    declare
        project_creator uuid;
        current_user_id uuid = current_user::uuid;
    begin
        select created_by from project where id = project_id_to_check into project_creator;

        return project_creator = current_user_id;
    end;
$$;

create or replace function current_user_is_staff_or_project_creator(
    project_id_to_check uuid
) returns boolean language plpgsql volatile strict as $$
    begin
        return current_user_is_staff() or current_user_is_project_creator(project_id_to_check);
    end;
$$;

alter table project_access enable row level security;
create policy anyone_can_see_project_membership on project_access for select using (true);
create policy staff_and_creator_can_add_others on project_access for insert with check (
    current_user_is_staff_or_project_creator(project_access.project_id)
);
create policy staff_and_creator_can_remove_others on project_access for delete using (
    current_user_is_staff_or_project_creator(project_access.project_id)
);

-- Enable row-level security. Note that because every view comes back to specimen, and specimen has
-- `security_invoker = true`, we don't need to enable it for anything else
alter table specimen enable row level security;
create policy only_project_members_can_see_specimen on specimen using (
    current_user_is_staff_or_project_creator(specimen.project_id)
    or current_user_has_access_to_project(specimen.project_id)
);
