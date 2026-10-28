select
    suspension_pool,
    array_agg(distinct(
        specimen, multiplexing_tag, null
    )::tagged_specimen) as specimens,
    array(
        select mes from suspension_pool_measurement as mes
        where
            mes.pool_id
            = (suspension_pool).id
    ) as measurements,
    array(
        select prep.prepared_by from suspension_pool_preparer as prep
        where prep.pool_id = (suspension_pool).id
    ) as preparers
from suspension_pool_to_specimen
/* {where} */
group by suspension_pool
