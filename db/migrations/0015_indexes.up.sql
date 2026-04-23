-- These indexes excludes fields with a small number of choices (like library type), fields that are `unique`, and
-- fields that aren't searched on

-- create index person_name on person (name);
-- create index person_created_at on person (created_at);
-- create index person_updated_at on person (updated_at);

-- create index project_started_at on projects (started_at);
-- create index project_ended_at on projects (ended_at);

-- create index project_member_person_id on project_member (person_id);
-- create index project_api_key_api_key_id on project_api_key

-- create index specimen_name on specimen (name);
-- create index specimen_submitted_by on specimen (submitted_by);
-- create index specimen_project_id on specimen (project_id);
-- create index specimen_received_at on specimen (received_at);
-- create index specimen_returned_at on specimen (returned_at);
-- create index specimen_returned_by on specimen (returned_by);
-- create index specimen_tissue on specimen (tissue);
-- create index specimen_additional_data on specimen using gin (additional_data);

-- create index single_index_sets_kit on single_index_set (kit);
-- create index single_index_sets_well on single_index_sets (well);

-- create index dual_index_sets_kit on dual_index_sets (kit);
-- create index dual_index_sets_well on dual_index_sets (well);
-- create index dual_index_sets_index_i7 on dual_index_sets (index_i7);
-- create index dual_index_sets_index2_workflow_a_i5 on dual_index_sets (index2_workflow_a_i5);
-- create index dual_index_sets_index2_workflow_b_i5 on dual_index_sets (index2_workflow_b_i5);

-- create index sequencing_runs_begun_at on sequencing_runs (begun_at);
-- create index sequencing_runs_finished_at on sequencing_runs (finished_at);
-- create index sequencing_runs_additional_data on sequencing_runs using gin (additional_data);

-- create index suspension_pools_project_id on suspension_pools (project_id);
-- create index suspension_pools_name on suspension_pools (name);
-- create index suspension_pools_pooled_at on suspension_pools (pooled_at);
-- create index suspension_pools_additional_data on suspension_pools using gin (additional_data);

-- create index suspensions_parent_specimen_id on suspensions (parent_specimen_id);
-- create index suspensions_project_id on suspensions (project_id);
-- create index suspensions_created_at on suspensions (created_at);
-- create index suspensions_lysis_duration_minutes on suspensions (lysis_duration_minutes);
-- create index suspensions_target_cell_recovery on suspensions (target_cell_recovery);
-- create index suspensions_additional_data on suspensions using gin (additional_data);

-- create index chromium_runs_project_id on chromium_runs (project_id);
-- create index chromium_runs_run_at on chromium_runs (run_at);
-- create index chromium_runs_run_by on chromium_runs (run_by);
-- create index chromium_runs_additional_data on chromium_runs using gin (additional_data);

-- create index gem_pools_chromium_run_id on gem_pools (chromium_run_id);

-- create index chip_loadings_gem_pool_id on chip_loadings (gem_pool_id);
-- create index chip_loadings_suspension_id on chip_loadings (suspension_id);
-- create index chip_loadings_suspension_pool_id on chip_loadings (suspension_pool_id);
-- create index chip_loadings_additional_data on chip_loadings using gin (additional_data);

-- create index cdna_prepared_at on cdna (prepared_at);
-- create index cdna_gem_pool_id on cdna (gem_pool_id);
-- create index cdna_project_id on cdna (project_id);
-- create index cdna_n_amplification_cycles on cdna (n_amplification_cycles);
-- create index cdna_additional_data on cdna using gin (additional_data);

-- create index libraries_cdna_id on libraries (cdna_id);
-- create index libraries_project_id on libraries (project_id);
-- create index libraries_single_index_set_name on libraries (single_index_set_name);
-- create index libraries_dual_index_set_name on libraries (dual_index_set_name);
-- create index libraries_number_of_sample_index_pcr_cycles on libraries (number_of_sample_index_pcr_cycles);
-- create index libraries_target_reads_per_cell on libraries (target_reads_per_cell);
-- create index libraries_prepared_at on libraries (prepared_at);
-- create index libraries_additional_data on libraries using gin (additional_data);

-- create index chromium_datasets_name on chromium_datasets (name);
-- create index chromium_datasets_project_id on chromium_datasets (project_id);
-- create index chromium_datasets_delivered_at on chromium_datasets (delivered_at);

-- create index chromium_dataset_metrics_files_dataset_id on chromium_dataset_metrics_files (dataset_id);
-- create index chromium_dataset_metrics_files_directory on chromium_dataset_metrics_files (directory);
-- create index chromium_dataset_metrics_files_filename on chromium_dataset_metrics_files (filename);
-- create index chromium_dataset_metrics_files_content_type on chromium_dataset_metrics_files (content_type);

-- create index chromium_dataset_web_summaries_dataset_id on chromium_dataset_web_summaries (dataset_id);
-- create index chromium_dataset_web_summaries_directory on chromium_dataset_web_summaries (directory);
-- create index chromium_dataset_web_summaries_filename on chromium_dataset_web_summaries (filename);

select 1;
