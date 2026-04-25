create view suspension_pool_brief as (
    select
        *,
        row('/suspension-pools/' || id)::simple_links as links
    from suspension_pool
);

create type suspension_brief_tagged_or_ocm_barcoded as (
    suspension suspension_brief,
    tag multiplexing_tag,
    ocm_barcode_id text
);

create view suspension_pool_full as (
    select
        pool as suspension_pool,
        array(
            select mes from suspension_pool_measurement as mes
            where mes.pool_id = pool.id
        ) as measurements,
        array(
            select prep.prepared_by
            from suspension_pool_preparer as prep
            where prep.pool_id = pool.id
        ) as preparers,
        array(
            -- We know that the individual suspensions weren't multiplexed on-chip because they are pooled
            select (susp, tag, null)::suspension_brief_tagged_or_ocm_barcoded
            from suspension_pooling as pooling
            join suspension_brief as susp on pooling.suspension_id = susp.id
            left join multiplexing_tag as tag on pooling.tag_id = tag.id
            where pooling.pool_id = pool.id
        ) as suspensions
    from suspension_pool_brief as pool
);
