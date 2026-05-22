select distinct on ((cdna).id)
    cdna,
    array_agg((
        specimen, multiplexing_tag, ocm_barcode_id
    )::tagged_specimen) as specimens,
    array(
        select m from cdna_measurement as m
        where m.cdna_id = (cdna).id
    ) as measurements,
    array(
        select prep.prepared_by from cdna_preparer as prep
        where prep.cdna_id = (cdna).id
    ) as preparers
from cdna_to_specimen
where true
group by cdna
