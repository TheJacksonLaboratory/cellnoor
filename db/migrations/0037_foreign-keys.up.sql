-- Prior to this migration, the following situation could occur:
-- 1. A project with ended_at = null is inserted
-- 2. A specimen which references that project is inserted. Through the trigger defined in
-- db/migrations/0004_specimen.up.sql, it gets the project's started_at and ended_at
--
-- Because project_ended_at is null, Postgres will not enforce the foreign key constraint on line 27 of the
-- aforementioned migration. As such, when the project is updated, the updates do not cascade to the specimen. The same
-- issue occurs with a specimen and its parent specimen, through the host_species column. This migration fixes these
-- bugs by grouping each nullable column with a non-null column into a composite type: (started_at, ended_at) becomes
-- project_active_period, and (species, host_species) becomes specimen_species. Postgres skips a foreign key only when
-- one of its columns is null, and a composite value is not null even when one of its fields is. When comparing two
-- composite values, Postgres also treats two null fields as equal, so a specimen's copy of a project period with no end
-- still matches its project, and updates to the project cascade to it.

create type project_active_period as (started_at timestamptz, ended_at timestamptz);

alter table specimen drop constraint specimen_project_id_project_started_at_project_ended_at_fkey;
alter table specimen drop constraint received_after_project_start;
alter table specimen drop constraint received_before_project_end;
alter table project drop constraint project_id_started_at_ended_at_key;
alter table project drop constraint starts_before_ends;

alter table project add active_period project_active_period;
update project set active_period = (started_at, ended_at);
alter table project alter active_period set not null;
alter table project drop started_at, drop ended_at;
alter table project add constraint started_at_not_null check ((active_period).started_at is not null);
alter table project add constraint starts_before_ends check ((active_period).started_at <= (active_period).ended_at);
alter table project add unique (id, active_period);

create index project_started_at_idx on project (((active_period).started_at));
create index project_ended_at_idx on project (((active_period).ended_at));

alter table specimen add project_active_period project_active_period;
update specimen set project_active_period = project.active_period from project
where specimen.project_id = project.id;
alter table specimen alter project_active_period set not null;
alter table specimen drop project_started_at, drop project_ended_at;

alter table specimen add constraint received_after_project_start check (
    received_or_created_at >= (project_active_period).started_at
);
alter table specimen add constraint received_before_project_end check (
    received_or_created_at <= (project_active_period).ended_at
);
alter table specimen add foreign key (project_id, project_active_period) references project (
    id, active_period
) on update cascade;

create or replace function populate_specimen_project_timestamps() returns trigger language plpgsql as $$
    begin
        select active_period into new.project_active_period from project where id = new.project_id;

        return new;
    end;
$$;

create type specimen_species as (species case_insensitive_text, host_species case_insensitive_text);

alter table specimen drop constraint specimen_parent_specimen_id_project_id_species_tissue_fkey;
alter table specimen drop constraint specimen_parent_specimen_id_host_species_fkey;
alter table specimen drop constraint specimen_id_project_id_species_tissue_key;
alter table specimen drop constraint specimen_id_host_species_key;
alter table specimen drop constraint host_species_different_from_donor_species;
drop index specimen_species_idx;

alter table specimen alter species type specimen_species using (species, host_species);
alter table specimen drop column host_species;

with recursive lineage as (
    select
        id,
        species
    from specimen
    where parent_specimen_id is null
    union all
    select
        specimen.id,
        lineage.species
    from specimen inner join lineage on specimen.parent_specimen_id = lineage.id
)

update specimen set species = lineage.species
from lineage
where specimen.id = lineage.id;

alter table specimen add constraint species_not_null check ((species).species is not null);
alter table specimen add constraint host_species_different_from_donor_species check (
    (species).species != (species).host_species
);
alter table specimen add unique (id, project_id, species, tissue);
alter table specimen add foreign key (parent_specimen_id, project_id, species, tissue) references specimen (
    id, project_id, species, tissue
) on update cascade;

create index specimen_species_idx on specimen (((species).species));
create index specimen_host_species_idx on specimen (((species).host_species));

create or replace function populate_specimen_inherited_fields() returns trigger language plpgsql as $$
    begin
        if new.parent_specimen_id is null then
            new.parent_specimen_received_or_created_at := null;
            return new;
        end if;

        select
            received_or_created_at,
            project_id,
            project_active_period,
            species,
            tissue
        into
            new.parent_specimen_received_or_created_at,
            new.project_id,
            new.project_active_period,
            new.species,
            new.tissue
        from specimen where id = new.parent_specimen_id;

        return new;
    end;
$$;

alter table specimen rename returned_at to given_at;
alter table specimen rename returned_by to given_by;
alter table specimen add given_to uuid references person;

alter table specimen rename constraint specimen_returned_by_fkey to specimen_given_by_fkey;
alter table specimen rename constraint received_before_returned to received_before_given;
alter index specimen_returned_at_idx rename to specimen_given_at_idx;
alter index specimen_returned_by_idx rename to specimen_given_by_idx;
create index specimen_given_to_idx on specimen (given_to);

-- A derived specimen has no submitter, so it was given to the submitter of its primary specimen, which is the last
-- element of `specimen_detailed.ancestors`
update specimen set
    given_to
    = coalesce(
        (specimen_detailed.specimen).submitted_by,
        (specimen_detailed.ancestors[cardinality(specimen_detailed.ancestors)]).submitted_by
    )
from specimen_detailed
where specimen.id = (specimen_detailed.specimen).id and specimen.given_at is not null;

alter table specimen drop constraint return_is_fully_specified;
alter table specimen add constraint given_is_fully_specified check (num_nulls(given_at, given_by, given_to) in (0, 3));
