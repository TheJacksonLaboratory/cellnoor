select
    cdna,
    array_agg(distinct(
        specimen, multiplexing_tag, ocm_barcode_id
    )::tagged_specimen) as specimens,
    array(
        select mes from cdna_measurement as mes
        where mes.cdna_id = (cdna).id
    ) as measurements,
    array(
        select prep.prepared_by from cdna_preparer as prep
        where prep.cdna_id = (cdna).id
    ) as preparers
from chromium_cdna_to_specimen
/* {where} */
group by cdna
