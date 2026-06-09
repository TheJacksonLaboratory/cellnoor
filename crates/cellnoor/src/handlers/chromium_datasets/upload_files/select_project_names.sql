select
    (cd.chromium_dataset).name as dataset_name,
    array_agg(proj.name) as project_names
from chromium_dataset_to_specimen as cd
join project as proj on (cd.specimen).project_id = proj.id
where (cd.chromium_dataset).id = $1
group by (cd.chromium_dataset);
