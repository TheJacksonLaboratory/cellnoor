create view chromium_run_to_assay as (
    select
        chr as chromium_run,
        assay
    from chromium_run as chr join tenx_assay as assay on chr.assay_id = assay.id
);

-- This view is complex because from a gem_pool, we can get to a specimen either from a suspension (query 1) or from a
-- suspension_pool (query 2)
create view gem_pool_to_specimen as (
    select
        chr as chromium_run,
        chip as chip_loading,
        susp.specimen,
        gp as gem_pool,
        susp as suspension,
        null as suspension_pool
    from chip_loading as chip
    join gem_pool as gp on chip.gem_pool_id = gp.id
    join chromium_run_to_assay as chr on gp.chromium_run_id = (chr.chromium_run).id
    join suspension_to_specimen as susp on chip.suspension_id = (susp.suspension).id

    -- `union all` because we don't need deduplication because we know there are no duplicates
    union all

    select
        chr as chromium_run,
        chip as chip_loading,
        pool.specimen,
        gp as gem_pool,
        null as suspension,
        pool
    from chip_loading as chip
    join gem_pool as gp on chip.gem_pool_id = gp.id
    join chromium_run_to_assay as chr on gp.chromium_run_id = (chr.chromium_run).id
    join suspension_pool_to_specimen as pool on chip.suspension_pool_id = (pool.suspension_pool).id
);

create function get_chromium_run_at_from_gem_pool_id(
    gem_pool_id uuid
) returns timestamptz language plpgsql volatile strict as $$
    begin
        return (select chr.run_at from gem_pool as gp join chromium_run as chr on gp.chromium_run_id = chr.id where gp.id = gem_pool_id);
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
