select
    library,
    array_agg(distinct(
        specimen, multiplexing_tag, ocm_barcode_id
    )::tagged_specimen) as specimens,
    array(
        select mes from library_measurement as mes
        where mes.library_id = (library).id
    ) as measurements,
    array(
        select prep.prepared_by from library_preparer as prep
        where prep.library_id = (library).id
    ) as preparers
from chromium_library_to_specimen
/* {where} */
group by library
