create view library_to_specimen as (
    select
        lib as library,
        cdna.specimen,
        cdna
    from library as lib join cdna_to_specimen as cdna on lib.cdna_id = (cdna.cdna).id
);
