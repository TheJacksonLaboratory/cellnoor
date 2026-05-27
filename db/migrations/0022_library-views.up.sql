create view library_to_specimen as (
    select
        library,
        -- Bring the following columns forward because they're useful
        cdna_ts as cdna,
        cdna_ts.specimen,
        cdna_ts.tenx_assay,
        cdna_ts.multiplexing_tag,
        cdna_ts.ocm_barcode_id
    from library join cdna_to_specimen as cdna_ts on library.cdna_id = (cdna_ts.cdna).id
);

create function check_libraries_from_same_gem_well() returns trigger language plpgsql volatile strict as $$
    declare
        n_gem_wells integer;
    begin
        select count(distinct (cdna.cdna.gem_well_id)) from library_to_specimen where (library.id) == new.library_id into n_gem_wells;
        if (n_gem_wells != 1) then
            raise check_violation using message = 'all libraries in Chromium dataset must come from the same GEM well';
        end if;

        return new;
    end;
$$;

create trigger libraries_from_same_gem_well before insert or update on chromium_dataset_library for each row execute function check_libraries_from_same_gem_well();
