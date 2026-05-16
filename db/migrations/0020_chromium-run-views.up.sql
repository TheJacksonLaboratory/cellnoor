-- This view is complex because from a gem_pool, we can get to a specimen either from a suspension (query 1) or from a
-- suspension_pool (query 2)
create view gem_pool_to_specimen as (
    select
        chromium_run,
        tenx_assay,
        suspension.specimen,
        gem_pool,
        chip_loading.ocm_barcode_id,
        null as multiplexing_tag
    from chip_loading
    join gem_pool on chip_loading.gem_pool_id = gem_pool.id
    join chromium_run on gem_pool.chromium_run_id = chromium_run.id
    join tenx_assay on chromium_run.assay_id = tenx_assay.id
    join suspension_to_specimen as suspension on chip_loading.suspension_id = (suspension.suspension).id

    -- `union all` because we don't need deduplication because we know there are no duplicates
    union all

    select
        chromium_run,
        tenx_assay,
        suspension_pool.specimen,
        gem_pool,
        chip_loading.ocm_barcode_id,
        suspension_pool.multiplexing_tag
    from chip_loading
    join gem_pool on chip_loading.gem_pool_id = gem_pool.id
    join chromium_run on gem_pool.chromium_run_id = chromium_run.id
    join tenx_assay on chromium_run.assay_id = tenx_assay.id
    join
        suspension_pool_to_specimen as suspension_pool
        on chip_loading.suspension_pool_id = (suspension_pool.suspension_pool).id
);

create type gem_pool_with_specimens as (
    gem_pool gem_pool,
    specimens tagged_specimen[]
);

create function get_chromium_run_at_from_gem_pool_id(
    gem_pool_id uuid
) returns timestamptz language plpgsql volatile strict as $$
    begin
        return (select chromium_run.run_at from gem_pool join chromium_run on gem_pool.chromium_run_id = chromium_run.id where gem_pool.id = gem_pool_id);
    end;
$$;

create function check_chromium_run_after_loaded_items() returns trigger language plpgsql volatile strict as $$
    declare
        chromium_run_at timestamptz = get_chromium_run_at_from_gem_pool_id(new.gem_pool_id);
        suspension_created_at timestamptz = get_suspension_created_at(new.suspension_id);
        suspension_pool_pooled_at timestamptz;
    begin
        select pooled_at from suspension_pool where id = new.suspension_pool_id into suspension_pool_pooled_at;

        if (greatest(suspension_created_at, suspension_pool_pooled_at) > chromium_run_at) then
            raise check_violation using message = 'Chromium run cannot occur before its constituent suspensions and/or suspension pools', table = tg_table_name;
        end if;

        return new;
    end;
$$;

create trigger chromium_run_after_loaded_items before insert or update on chip_loading for each row execute function check_chromium_run_after_loaded_items();
