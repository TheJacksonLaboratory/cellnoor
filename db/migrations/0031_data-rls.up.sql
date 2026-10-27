-- `security definer` so that it reads `project` and `project_access` without their policies, which call this function
-- and would otherwise recurse into each other
create function current_user_is_project_member(
    project_id_to_check uuid
) returns boolean language sql stable security definer as $$
    select
        exists (select 1 from project where id = project_id_to_check and created_by = app_user_id())
        or exists (
            select 1 from project_access where project_id = project_id_to_check and principal_id = app_user_id()
        )
$$;

-- A permission is granted on a resource, which is a group of tables
create temp table resource_table (resource text not null, table_name text primary key);

insert into resource_table (resource, table_name) values
('institution', 'institution'),
('person', 'person'),
('account', 'account'),
('project', 'project'),
('project', 'project_access'),
('specimen', 'specimen'),
('specimen', 'committee_approval'),
('specimen', 'specimen_measurement'),
('assay_constant_data', 'tenx_assay'),
('assay_constant_data', 'index_kit'),
('assay_constant_data', 'single_index_set'),
('assay_constant_data', 'dual_index_set'),
('assay_constant_data', 'library_type_specification'),
('assay_constant_data', 'multiplexing_tag'),
('chromium_experimental_data', 'suspension'),
('chromium_experimental_data', 'suspension_measurement'),
('chromium_experimental_data', 'suspension_preparer'),
('chromium_experimental_data', 'suspension_pool'),
('chromium_experimental_data', 'suspension_pooling'),
('chromium_experimental_data', 'suspension_pool_measurement'),
('chromium_experimental_data', 'suspension_pool_preparer'),
('chromium_experimental_data', 'chromium_run'),
('chromium_experimental_data', 'gem_well'),
('chromium_experimental_data', 'chip_loading'),
('chromium_experimental_data', 'cdna'),
('chromium_experimental_data', 'cdna_measurement'),
('chromium_experimental_data', 'cdna_preparer'),
('chromium_experimental_data', 'library'),
('chromium_experimental_data', 'library_measurement'),
('chromium_experimental_data', 'library_preparer'),
('chromium_dataset', 'chromium_dataset'),
('chromium_dataset', 'chromium_dataset_raw_file'),
('chromium_dataset', 'chromium_dataset_parsed_file'),
('chromium_dataset', 'chromium_dataset_library');


do $$
    declare
        r record;
        unmapped text;
    begin
        -- A new table with no resource would silently have no policies, so refuse to migrate until it has one.
        -- The remaining identity tables get their policies in the previous migrations, and `schema_migrations`
        -- belongs to the migration tool
        select string_agg(tablename, ', ') into unmapped from pg_tables
        where
            schemaname = 'public'
            and tablename not in (
                'schema_migrations', 'principal', 'permission', 'service', 'service_access', 'api_key'
            )
            and tablename not in (select table_name from resource_table);

        if unmapped is not null then
            raise exception 'no resource is defined for these tables: %', unmapped;
        end if;

        for r in select resource, table_name from resource_table
        loop
            execute format('alter table %I enable row level security', r.table_name);
            execute format('create policy anyone_can_read on %I for select using (true)', r.table_name);
            execute format(
                'create policy staff_can_create on %I for insert with check (current_user_is_staff() and current_user_can(%L, %L))',
                r.table_name, 'create', r.resource
            );
            execute format(
                'create policy staff_can_update on %I for update using (current_user_is_staff() and current_user_can(%L, %L))',
                r.table_name, 'update', r.resource
            );
            execute format(
                'create policy staff_can_delete on %I for delete using (current_user_is_staff() and current_user_can(%L, %L))',
                r.table_name, 'delete', r.resource
            );
        end loop;
    end;
$$;

-- The admin may do anything to every resource
-- noqa: disable=AL03
insert into permission (principal_id, resource, action)
select
    uuid_nil(),
    resource,
    action
from (select distinct resource from resource_table) as resources
cross join unnest(array['create', 'update', 'delete']) as action;
-- noqa: enable=AL03

drop table resource_table;

-- Non-staff only see the projects they are members of, and the specimens in them. Every other entity is only ever read
-- through a view that joins it to its specimens (db/migrations/0018_suspension-views.up.sql onwards), so scoping
-- specimens scopes everything downstream of them

-- `insert ... returning` checks the new project against this policy, and `current_user_is_project_member` can't see a
-- row inserted by the statement calling it, so the creator is checked here too
create policy members_only on project as restrictive for select using (
    created_by = app_user_id() or current_user_is_staff() or current_user_is_project_member(id)
);
create policy members_only on project_access as restrictive for select using (
    current_user_is_staff() or current_user_is_project_member(project_id)
);
create policy members_only on specimen as restrictive for select using (
    current_user_is_staff() or current_user_is_project_member(project_id)
);
create policy members_only on specimen_measurement as restrictive for select using (
    exists (select 1 from specimen where id = specimen_id)
);
create policy members_only on committee_approval as restrictive for select using (
    exists (select 1 from specimen where id = specimen_id)
);

-- What non-staff may write

create policy anyone_can_create on project for insert with check (current_user_can('create', 'project'));
create policy creator_is_current_user on project as restrictive for insert with check (created_by = app_user_id());

create policy members_can_add_members on project_access for insert with check (
    current_user_is_project_member(project_id)
);
create policy creator_can_remove_members on project_access for delete using (
    exists (select 1 from project where id = project_id and created_by = app_user_id())
);

create policy members_can_create on specimen for insert with check (
    current_user_can('create', 'specimen') and current_user_is_project_member(project_id)
);
create policy members_can_create on specimen_measurement for insert with check (
    current_user_can('create', 'specimen') and exists (select 1 from specimen where id = specimen_id)
);
create policy members_can_create on committee_approval for insert with check (
    current_user_can('create', 'specimen') and exists (select 1 from specimen where id = specimen_id)
);
