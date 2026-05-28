select count(distinct cdna.gem_well_id)
from library
join cdna on library.cdna_id = cdna.id
where library.id = any($1)
