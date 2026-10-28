select
    count(distinct cdna.gem_well_id) as n_gem_wells,
    count(distinct cdna.library_type) as n_library_types
from library
join cdna on library.cdna_id = cdna.id
where library.id = any($1)
