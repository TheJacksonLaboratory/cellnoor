create view library_compact as (
    select
        *,
        row('/libraries/' || id)::simple_links as links
    from library
);

create view library_to_specimen as (
    select
        lib as library,
        cdna.specimen,
        cdna
    from library_compact as lib join cdna_to_specimen as cdna on lib.cdna_id = (cdna.cdna).id
);
