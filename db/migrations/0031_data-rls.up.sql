-- Staff see every project, and so everything descending from one. Everyone else sees only the projects they were
-- given access to
create function current_user_has_access_to_project(project_id_to_check uuid) returns boolean language sql stable as $$
    select current_user_is_staff() or exists (
        select 1 from project_access where project_id = project_id_to_check and principal_id = app_user_id()
    )
$$;

-- A permission is granted on a resource, which is a group of tables. Reads are open except for projects and
-- specimens, which are scoped to their members. Every other view comes back to specimen through a view with
-- `security_invoker = true`, so scoping specimens scopes everything downstream of them
create temp table resource_table (resource text not null, table_name text primary key);

insert into resource_table (resource, table_name) values
('institution', 'institution'),
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
        -- Which rows of this table the user can see, which is also which rows they can update or delete
        visible_rows text;
        unmapped text;
    begin
        -- A new table with no resource would silently have no policies, so refuse to migrate until it has one.
        -- The identity tables get their policies in the previous migrations, and `schema_migrations` belongs to the
        -- migration tool
        select string_agg(tablename, ', ') into unmapped from pg_tables
        where
            schemaname = 'public'
            and tablename not in (
                'schema_migrations', 'principal', 'permission', 'person', 'account', 'service', 'service_access', 'api_key'
            )
            and tablename not in (select table_name from resource_table);

        if unmapped is not null then
            raise exception 'no resource is defined for these tables: %', unmapped;
        end if;

        for r in select resource, table_name from resource_table
        loop
            visible_rows = case r.table_name
                when 'project' then 'current_user_has_access_to_project(id)'
                when 'specimen' then 'current_user_has_access_to_project(project_id)'
                else 'true'
            end;

            execute format('alter table %I enable row level security', r.table_name);
            execute format(
                'create policy can_read on %I for select using (%s)', r.table_name, visible_rows
            );
            execute format(
                'create policy can_create on %I for insert with check (current_user_can(%L, %L))',
                r.table_name, 'create', r.resource
            );
            execute format(
                'create policy can_update on %I for update using (%s) with check (current_user_can(%L, %L))',
                r.table_name, visible_rows, 'update', r.resource
            );
            execute format(
                'create policy can_delete on %I for delete using (%s and current_user_can(%L, %L))',
                r.table_name, visible_rows, 'delete', r.resource
            );
        end loop;
    end;
$$;

drop table resource_table;

-- `created_by` defaults to the current user, and this stops anyone from creating a project as someone else
create policy creator_is_current_user on project as restrictive for insert with check (created_by = app_user_id());
