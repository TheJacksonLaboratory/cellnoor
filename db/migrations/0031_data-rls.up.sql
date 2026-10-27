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

-- A permission is granted on a resource, which is a group of tables. A child table holds rows that hang off an entity,
-- like measurements and preparers, so whoever may update the entity may also insert and delete them
create temp table resource_table (
    resource text primary key, entity_tables text[] not null, child_tables text[] not null
);

insert into resource_table (resource, entity_tables, child_tables) values
('institution', '{institution}', '{}'),
('person', '{person}', '{}'),
('account', '{account}', '{}'),
('project', '{project}', '{project_access}'),
('specimen', '{specimen}', '{specimen_measurement, committee_approval}'),
(
    'assay_constant_data',
    '{tenx_assay, index_kit, single_index_set, dual_index_set, library_type_specification, multiplexing_tag}',
    '{}'
),
(
    'chromium_experimental_data',
    '{suspension, suspension_pool, chromium_run, cdna, library}',
    '{
        suspension_measurement, suspension_preparer,
        suspension_pooling, suspension_pool_measurement, suspension_pool_preparer,
        gem_well, chip_loading,
        cdna_measurement, cdna_preparer,
        library_measurement, library_preparer
    }'
),
(
    'chromium_dataset',
    '{chromium_dataset}',
    '{chromium_dataset_raw_file, chromium_dataset_parsed_file, chromium_dataset_library}'
);

-- Everyone reads, and staff write with the permission. The tables skipped here have rules for non-staff too, so their
-- policies are written out after this
do $$
    declare
        r record;
    begin
        for r in
            select
                resource,
                table_name,
                table_name = any(child_tables) as is_child
            from resource_table, unnest(entity_tables || child_tables) as table_name
        loop
            continue when r.table_name in (
                'project', 'project_access', 'specimen', 'specimen_measurement', 'committee_approval'
            );

            execute format(
                $policies$
                    alter table %1$I enable row level security;
                    create policy can_read on %1$I for select using (true);
                    create policy can_create on %1$I for insert with check (
                        current_user_is_staff() and (current_user_can('create', %2$L) %3$s)
                    );
                    create policy can_update on %1$I for update using (
                        current_user_is_staff() and current_user_can('update', %2$L)
                    );
                    create policy can_delete on %1$I for delete using (
                        current_user_is_staff() and (current_user_can('delete', %2$L) %3$s)
                    );
                $policies$,
                r.table_name,
                r.resource,
                case when r.is_child then format('or current_user_can(%L, %L)', 'update', r.resource) else '' end
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
from resource_table
cross join unnest(array['create', 'update', 'delete']) as action;
-- noqa: enable=AL03

drop table resource_table;

-- Non-staff only see the projects they are members of and the specimens in them. Every other entity is only ever read
-- through a view that joins it to its specimens (db/migrations/0018_suspension-views.up.sql onwards), so scoping
-- specimens scopes everything downstream of them
alter table project enable row level security;

-- `insert ... returning` checks the new project against this policy, and `current_user_is_project_member` can't see a
-- row inserted by the statement calling it, so the creator is checked here too
create policy can_read on project for select using (
    current_user_is_staff() or created_by = app_user_id() or current_user_is_project_member(id)
);
create policy can_create on project for insert with check (
    current_user_can('create', 'project') and created_by = app_user_id()
);
create policy can_update on project for update using (
    current_user_is_staff() and current_user_can('update', 'project')
);
create policy can_delete on project for delete using (
    current_user_is_staff() and current_user_can('delete', 'project')
);

alter table project_access enable row level security;

-- Any member may add people to a project, but only its creator may remove them
create policy can_read on project_access for select using (
    current_user_is_staff() or current_user_is_project_member(project_id)
);
create policy can_create on project_access for insert with check (
    (current_user_is_staff() and (current_user_can('create', 'project') or current_user_can('update', 'project')))
    or current_user_is_project_member(project_id)
);
create policy can_update on project_access for update using (
    current_user_is_staff() and current_user_can('update', 'project')
);
create policy can_delete on project_access for delete using (
    (current_user_is_staff() and (current_user_can('delete', 'project') or current_user_can('update', 'project')))
    or exists (
        select 1 from project
        where id = project_id and created_by = app_user_id()
    )
);

alter table specimen enable row level security;

create policy can_read on specimen for select using (
    current_user_is_staff() or current_user_is_project_member(project_id)
);
create policy can_create on specimen for insert with check (
    current_user_can('create', 'specimen') and (current_user_is_staff() or current_user_is_project_member(project_id))
);
create policy can_update on specimen for update using (
    current_user_is_staff() and current_user_can('update', 'specimen')
);
create policy can_delete on specimen for delete using (
    current_user_is_staff() and current_user_can('delete', 'specimen')
);

-- `exists (select 1 from specimen ...)` is subject to the policy on `specimen`, so it only finds specimens the user can
-- see

alter table specimen_measurement enable row level security;

create policy can_read on specimen_measurement for select using (
    exists (
        select 1 from specimen
        where id = specimen_id
    )
);
create policy can_create on specimen_measurement for insert with check (
    (current_user_is_staff() and (current_user_can('create', 'specimen') or current_user_can('update', 'specimen')))
    or ((current_user_can('create', 'specimen') or current_user_can('update', 'specimen')) and exists (
        select 1 from specimen
        where id = specimen_id
    ))
);
create policy can_update on specimen_measurement for update using (
    current_user_is_staff() and current_user_can('update', 'specimen')
);
create policy can_delete on specimen_measurement for delete using (
    current_user_is_staff() and (current_user_can('delete', 'specimen') or current_user_can('update', 'specimen'))
);

alter table committee_approval enable row level security;

create policy can_read on committee_approval for select using (
    exists (
        select 1 from specimen
        where id = specimen_id
    )
);
create policy can_create on committee_approval for insert with check (
    (current_user_is_staff() and (current_user_can('create', 'specimen') or current_user_can('update', 'specimen')))
    or ((current_user_can(
        'create', 'specimen') or current_user_can(
        'update', 'specimen'
    )) and exists (
        select 1 from specimen
        where id = specimen_id
    ))
);
create policy can_update on committee_approval for update using (
    current_user_is_staff() and current_user_can('update', 'specimen')
);
create policy can_delete on committee_approval for delete using (
    current_user_is_staff() and (current_user_can('delete', 'specimen') or current_user_can('update', 'specimen'))
);
