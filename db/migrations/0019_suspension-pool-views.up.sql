-- We include the multiplexing tag and OCM barcode because it's cheap and is useful for consumers
create type tagged_specimen as (
    specimen specimen,
    multiplexing_tag multiplexing_tag,
    ocm_barcode_id case_insensitive_text
);

create view suspension_pool_to_specimen with (security_invoker = true) as (
    select
        suspension_pool,
        suspension.specimen,
        suspension,
        multiplexing_tag
    from suspension_pool
    join suspension_pooling as pooling on suspension_pool.id = pooling.pool_id
    join suspension_to_specimen as suspension on pooling.suspension_id = (suspension.suspension).id
    left join multiplexing_tag on (pooling.tag_id, pooling.tag_type) = (multiplexing_tag.tag_id, multiplexing_tag.type)
);
