-- Only grant select on the tables and views a user accesses directly to prevent the developer from forgetting that
-- there are convenient views already made
grant select on institution,
person_public,
service,
service_access,
api_key_public,
project,
project_access,
project_detailed,
specimen,
specimen_measurement,
specimen_detailed,
tenx_assay,
index_kit,
single_index_set,
dual_index_set,
library_type_specification,
suspension,
suspension_detailed,
suspension_pool_to_specimen,
suspension_pool_measurement,
suspension_pool_preparer,
gem_well_to_specimen,
chromium_cdna_to_specimen,
cdna_preparer,
cdna_measurement,
chromium_library_to_specimen,
library_preparer,
library_measurement,
chromium_dataset_to_specimen,
chromium_dataset_raw_file,
chromium_dataset_parsed_file to public;

grant insert (description, owned_by, is_staff, can_manage_users), update (description, owned_by, is_staff, can_manage_users), delete on service to public;

grant insert, delete on service_access to public;

grant insert (description, hashed_key, person_id, service_id, expires_at),
select (id, description, person_id, service_id, created_at, expires_at),
update (description, expires_at),
delete on api_key to public;
