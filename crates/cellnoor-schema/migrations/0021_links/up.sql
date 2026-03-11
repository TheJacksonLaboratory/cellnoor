alter table institutions add column members text generated always as (
    '/institutions/' || id || '/members'
) stored not null;
alter table institutions drop column links;

alter table people add column projects text generated always as ('/people/' || id || '/projects') stored not null;
alter table people add column specimens text generated always as ('/people/' || id || '/specimens') stored not null;
alter table people drop column links;

alter table projects add column people text generated always as ('/projects/' || id || '/people') stored not null;
alter table projects add column specimens text generated always as ('/projects/' || id || '/specimens') stored not null;
alter table projects add column chromium_datasets text generated always as (
    '/projects/' || id || '/chromium_datasets'
) stored not null;
alter table projects drop column links;

alter table specimens add column measurements text generated always as (
    '/specimens/' || id || '/measurements'
) stored not null;
alter table specimens add column suspensions text generated always as (
    '/specimens/' || id || '/suspensions'
) stored not null;
alter table specimens add column chromium_datasets text generated always as (
    '/specimens/' || id || '/chromium_datasets'
) stored not null;
alter table specimens drop column links;

alter table tenx_assays drop column links;

alter table sequencing_runs add column libraries text generated always as (
    '/sequencing-runs/' || id || '/libraries'
) stored not null;
alter table sequencing_runs drop column links;

alter table suspension_pools add column measurements text generated always as (
    '/suspension-pools/' || id || '/measurements'
) stored not null;
alter table suspension_pools add column suspensions text generated always as (
    '/suspension-pools/' || id || '/suspensions'
) stored not null;
alter table suspension_pools drop column links;

alter table suspensions add column measurements text generated always as (
    '/suspensions/' || id || '/measurements'
) stored not null;
alter table suspensions drop column links;

alter table chromium_runs drop column links;

alter table gem_pools drop column links;

alter table cdna add column measurements text generated always as ('/cdna/' || id || '/measurements') stored not null;
alter table cdna add column libraries text generated always as ('/cdna/' || id || '/libraries') stored not null;
alter table cdna drop column links;

alter table libraries add column measurements text generated always as (
    '/libraries/' || id || '/measurements'
) stored not null;
alter table libraries add column sequencing_runs text generated always as (
    '/libraries/' || id || '/sequencing_runs'
) stored not null;
alter table libraries add column chromium_datasets text generated always as (
    '/libraries/' || id || '/chromium_datasets'
) stored not null;
alter table libraries drop column links;
