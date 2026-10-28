select
    (cds.chromium_dataset).name as dataset_name,
    array_agg(proj.name) as project_names
from chromium_dataset_to_specimen as cds
join project as proj on (cds.specimen).project_id = proj.id
where (cds.chromium_dataset).id = $1
group by (cds.chromium_dataset);
