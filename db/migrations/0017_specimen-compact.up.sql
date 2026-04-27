create view specimen_compact with (security_invoker = true) as (
    select
        *,
        row('/specimens/' || id)::simple_links as links
    from specimen
);
