create view cdna_to_specimen as (
    select
        cdna,
        gem_pool.specimen,
        gem_pool.tenx_assay
    from cdna join gem_pool_to_specimen as gem_pool on cdna.gem_pool_id = (gem_pool.gem_pool).id
);

create function check_cdna_prepared_after_chromium_run() returns trigger language plpgsql volatile strict as $$
    declare
        chromium_run_at timestamptz = get_chromium_run_at_from_gem_pool_id(new.gem_pool_id);
    begin
        if (chromium_run_at > new.prepared_at) then
            raise check_violation using message = 'cDNA cannot be prepared before the Chromium run it came from', table = tg_table_name;
        end if;

        return new;
    end;
$$;

create trigger cdna_prepared_after_chromium_run before insert or update on cdna for each row execute function check_cdna_prepared_after_chromium_run();
