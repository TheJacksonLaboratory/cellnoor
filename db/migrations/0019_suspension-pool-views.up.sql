-- We include the multiplexing tag because it's cheap and is useful for consumers
create view suspension_pool_to_specimen as (
    select
        pool as suspension_pool,
        susp.parent_specimen,
        susp as suspension,
        multiplexing_tag
    from suspension_pool as pool
    join suspension_pooling as pooling on pool.id = pooling.pool_id
    join suspension_to_specimen as susp on pooling.suspension_id = (susp.suspension).id
    left join multiplexing_tag on pooling.tag_id = multiplexing_tag.id
);
