create index person_name_idx on person (name);
create index person_institution_id_idx on person (institution_id);
create index person_created_at_idx on person (created_at);
create index person_updated_at_idx on person (updated_at);

create index service_description_idx on service (description);
create index service_owned_by_idx on service (owned_by);
create index service_created_at_idx on service (created_at);

create index service_access_person_id_idx on service_access (person_id);

create index api_key_description_idx on api_key (description);
create index api_key_person_id_idx on api_key (person_id);
create index api_key_service_id_idx on api_key (service_id);
create index api_key_created_at_idx on api_key (created_at);
create index api_key_expires_at_idx on api_key (expires_at);

create index project_created_by_person_idx on project (created_by_person);
create index project_created_by_service_idx on project (created_by_service);
create index project_started_at_idx on project (started_at);
create index project_ended_at_idx on project (ended_at);

create index project_access_person_id_idx on project_access (person_id);
create index project_access_service_id_idx on project_access (service_id);

create index specimen_name_idx on specimen (name);
create index specimen_submitted_by_idx on specimen (submitted_by);
create index specimen_project_id_idx on specimen (project_id);
create index specimen_received_at_idx on specimen (received_at);
create index specimen_returned_at_idx on specimen (returned_at);
create index specimen_returned_by_idx on specimen (returned_by);
create index specimen_tissue_idx on specimen (tissue);
create index specimen_additional_data_idx on specimen using gin (additional_data);

create index committee_approval_specimen_id_idx on committee_approval (specimen_id);

create index single_index_set_kit_idx on single_index_set (kit);
create index single_index_set_well_idx on single_index_set (well);
create index single_index_set_sequences_idx on single_index_set using gin (sequences);

create index dual_index_set_kit_idx on dual_index_set (kit);
create index dual_index_set_well_idx on dual_index_set (well);
create index dual_index_set_index_i7_idx on dual_index_set (index_i7);
create index dual_index_set_index2_workflow_a_i5_idx on dual_index_set (index2_workflow_a_i5);
create index dual_index_set_index2_workflow_b_i5_idx on dual_index_set (index2_workflow_b_i5);

create index suspension_specimen_id_idx on suspension (specimen_id);
create index suspension_created_at_idx on suspension (created_at);
create index suspension_lysis_duration_minutes_idx on suspension (lysis_duration_minutes);
create index suspension_target_cell_recovery_idx on suspension (target_cell_recovery);
create index suspension_additional_data_idx on suspension using gin (additional_data);

create index suspension_pool_name_idx on suspension_pool (name);
create index suspension_pool_pooled_at_idx on suspension_pool (pooled_at);
create index suspension_pool_additional_data_idx on suspension_pool using gin (additional_data);

create index suspension_pooling_suspension_id_idx on suspension_pooling (suspension_id);
create index suspension_pooling_tag_id_idx on suspension_pooling (tag_id);

create index chromium_run_assay_id_idx on chromium_run (assay_id);
create index chromium_run_run_at_idx on chromium_run (run_at);
create index chromium_run_run_by_idx on chromium_run (run_by);
create index chromium_run_additional_data_idx on chromium_run using gin (additional_data);

create index gem_well_chromium_run_id_idx on gem_well (chromium_run_id);

create index chip_loading_suspension_id_idx on chip_loading (suspension_id);
create index chip_loading_suspension_pool_id_idx on chip_loading (suspension_pool_id);
create index chip_loading_suspension_volume_loaded_idx on chip_loading using gin (suspension_volume_loaded);
create index chip_loading_buffer_volume_loaded_idx on chip_loading using gin (buffer_volume_loaded);
create index chip_loading_additional_data_idx on chip_loading using gin (additional_data);

create index cdna_prepared_at_idx on cdna (prepared_at);
create index cdna_n_amplification_cycles_idx on cdna (n_amplification_cycles);
create index cdna_additional_data_idx on cdna using gin (additional_data);

create index library_cdna_id_idx on library (cdna_id);
create index library_single_index_set_name_idx on library (single_index_set_name);
create index library_dual_index_set_name_idx on library (dual_index_set_name);
create index library_number_of_sample_index_pcr_cycles_idx on library (number_of_sample_index_pcr_cycles);
create index library_target_reads_per_cell_idx on library (target_reads_per_cell);
create index library_prepared_at_idx on library (prepared_at);
create index library_additional_data_idx on library using gin (additional_data);

create index chromium_dataset_name_idx on chromium_dataset (name);
create index chromium_dataset_delivered_at_idx on chromium_dataset (delivered_at);

create index chromium_dataset_library_library_id_idx on chromium_dataset_library (library_id);
