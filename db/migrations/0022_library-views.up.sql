create view library_to_specimen as (
    select
        library,
        cdna.specimen,
        cdna
    from library join cdna_to_specimen as cdna on library.cdna_id = (cdna.cdna).id
);
