alter table institutions drop column links;
alter table institutions add column self_link text generated always as ('/institutions/' || id) stored not null; -- noqa
alter table institutions add column members_link text generated always as (
    '/institutions/' || id || '/members'
) stored not null;

alter table people drop column links;
alter table people add column self_link text generated always as ('/people/' || id) stored not null;
alter table people add column projects_link text generated always as ('/people/' || id || '/projects') stored not null;
alter table people add column specimens_link text generated always as (
    '/people/' || id || '/specimens'
) stored not null;

alter table projects drop column links;
alter table projects add column self_link text generated always as ('/projects/' || id) stored not null;
alter table projects add column people_link text generated always as ('/projects/' || id || '/people') stored not null;
alter table projects add column specimens_link text generated always as (
    '/projects/' || id || '/specimens'
) stored not null;
alter table projects add column chromium_datasets_link text generated always as (
    '/projects/' || id || '/chromium-datasets'
) stored not null;

alter table specimens drop column links;
alter table specimens add column self_link text generated always as ('/specimens/' || id) stored not null;
alter table specimens add column measurements_link text generated always as (
    '/specimens/' || id || '/measurements'
) stored not null;
alter table specimens add column suspensions_link text generated always as (
    '/specimens/' || id || '/suspensions'
) stored not null;
alter table specimens add column chromium_datasets_link text generated always as (
    '/specimens/' || id || '/chromium_datasets'
) stored not null;

alter table tenx_assays drop column links;
alter table tenx_assays add column self_link text generated always as ('/10x-assays/' || id) stored not null;

alter table sequencing_runs drop column links;
alter table sequencing_runs add column self_link text generated always as ('/sequencing-runs/' || id) stored not null;
alter table sequencing_runs add column libraries_link text generated always as (
    '/sequencing-runs/' || id || '/libraries'
) stored not null;

alter table suspension_pools drop column links;
alter table suspension_pools add column self_link text generated always as ('/suspension-pools/' || id) stored not null;
alter table suspension_pools add column measurements_link text generated always as (
    '/suspension-pools/' || id || '/measurements'
) stored not null;
alter table suspension_pools add column suspensions_link text generated always as (
    '/suspension-pools/' || id || '/suspensions'
) stored not null;

alter table suspensions drop column links;
alter table suspensions add column self_link text generated always as ('/suspensions/' || id) stored not null;
alter table suspensions add column measurements_link text generated always as (
    '/suspensions/' || id || '/measurements'
) stored not null;

alter table chromium_runs drop column links;
alter table chromium_runs add column self_link text generated always as ('/chromium-runs/' || id) stored not null;

alter table gem_pools drop column links;
alter table gem_pools add column self_link text generated always as ('/gem-pools/' || id) stored not null;

alter table cdna drop column links;
alter table cdna add column self_link text generated always as ('/cdna/' || id) stored not null;
alter table cdna add column measurements_link text generated always as (
    '/cdna/' || id || '/measurements'
) stored not null;
alter table cdna add column libraries_link text generated always as ('/cdna/' || id || '/libraries') stored not null;

alter table libraries drop column links;
alter table libraries add column self_link text generated always as ('/libraries/' || id) stored not null;
alter table libraries add column measurements_link text generated always as (
    '/libraries/' || id || '/measurements'
) stored not null;
alter table libraries add column sequencing_runs_link text generated always as (
    '/libraries/' || id || '/sequencing-runs'
) stored not null;
alter table libraries add column chromium_datasets_link text generated always as (
    '/libraries/' || id || '/chromium-datasets'
) stored not null;

drop function construct_links;

alter table chromium_datasets drop column links;
alter table chromium_datasets add column self_link text generated always as (
    '/chromium-datasets/' || id
) stored not null;
alter table chromium_datasets add column specimens_link text generated always as (
    '/chromium-datasets/' || id || '/specimens'
) stored not null;
alter table chromium_datasets add column libraries_link text generated always as (
    '/chromium-datasets/' || id || '/libraries'
) stored not null;
alter table chromium_datasets add column file_links text [] default '{}' not null;
