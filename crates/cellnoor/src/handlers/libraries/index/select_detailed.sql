select distinct on ((library).id)
    library,
    array_agg((
        specimen, multiplexing_tag, ocm_barcode_id
    )::tagged_specimen) as specimens,
    array(
        select m from library_measurement as m
        where m.library_id = (library).id
    ) as measurements,
    array(
        select prep.prepared_by from library_preparer as prep
        where prep.library_id = (library).id
    ) as preparers
from library_to_specimen
/* {where} */
group by library
