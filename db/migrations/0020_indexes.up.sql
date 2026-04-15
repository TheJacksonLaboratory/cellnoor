-- Note: these indexes excludes fields with a small number of choices (like library type) and fields with `unique`

create index people_name_idx on people (name);
create index people_institution_id_idx on people (institution_id);

create index projects_started_at_idx on projects (started_at);
create index projects_ended_at_idx on projects (ended_at);

create index specimens_name_idx on specimens (name);
create index specimens_submitted_by_idx on specimens (submitted_by);
create index specimens_project_id_idx on specimens (project_id);
create index specimens_received_at_idx on specimens (received_at);
create index specimens_returned_at_idx on specimens (returned_at);
create index specimens_returned_by_idx on specimens (returned_by);
create index specimens_tissue_idx on specimens (tissue);
create index specimens_additional_data_idx on specimens using gin (additional_data);

create index single_index_sets_kit_idx on single_index_sets (kit);
create index single_index_sets_well_idx on single_index_sets (well);

create index dual_index_sets_kit_idx on dual_index_sets (kit);
create index dual_index_sets_well_idx on dual_index_sets (well);
create index dual_index_sets_index_i7_idx on dual_index_sets (index_i7);
create index dual_index_sets_index2_workflow_a_i5_idx on dual_index_sets (index2_workflow_a_i5);
create index dual_index_sets_index2_workflow_b_i5_idx on dual_index_sets (index2_workflow_b_i5);

create index sequencing_runs_begun_at_idx on sequencing_runs (begun_at);
create index sequencing_runs_finished_at_idx on sequencing_runs (finished_at);
create index sequencing_runs_additional_data_idx on sequencing_runs using gin (additional_data);

create index suspension_pools_project_id_idx on suspension_pools (project_id);
create index suspension_pools_name_idx on suspension_pools (name);
create index suspension_pools_pooled_at_idx on suspension_pools (pooled_at);
create index suspension_pools_additional_data_idx on suspension_pools using gin (additional_data);

create index suspensions_parent_specimen_id_idx on suspensions (parent_specimen_id);
create index suspensions_project_id_idx on suspensions (project_id);
create index suspensions_created_at_idx on suspensions (created_at);
create index suspensions_lysis_duration_minutes_idx on suspensions (lysis_duration_minutes);
create index suspensions_target_cell_recovery_idx on suspensions (target_cell_recovery);
create index suspensions_additional_data_idx on suspensions using gin (additional_data);

create index chromium_runs_project_id_idx on chromium_runs (project_id);
create index chromium_runs_run_at_idx on chromium_runs (run_at);
create index chromium_runs_run_by_idx on chromium_runs (run_by);
create index chromium_runs_additional_data_idx on chromium_runs using gin (additional_data);

create index gem_pools_chromium_run_id_idx on gem_pools (chromium_run_id);

create index chip_loadings_gem_pool_id_idx on chip_loadings (gem_pool_id);
create index chip_loadings_suspension_id_idx on chip_loadings (suspension_id);
create index chip_loadings_suspension_pool_id_idx on chip_loadings (suspension_pool_id);
create index chip_loadings_additional_data_idx on chip_loadings using gin (additional_data);

create index cdna_prepared_at_idx on cdna (prepared_at);
create index cdna_gem_pool_id_idx on cdna (gem_pool_id);
create index cdna_project_id_idx on cdna (project_id);
create index cdna_n_amplification_cycles_idx on cdna (n_amplification_cycles);
create index cdna_additional_data_idx on cdna using gin (additional_data);

create index libraries_cdna_id_idx on libraries (cdna_id);
create index libraries_project_id_idx on libraries (project_id);
create index libraries_single_index_set_name_idx on libraries (single_index_set_name);
create index libraries_dual_index_set_name_idx on libraries (dual_index_set_name);
create index libraries_number_of_sample_index_pcr_cycles_idx on libraries (number_of_sample_index_pcr_cycles);
create index libraries_target_reads_per_cell_idx on libraries (target_reads_per_cell);
create index libraries_prepared_at_idx on libraries (prepared_at);
create index libraries_additional_data_idx on libraries using gin (additional_data);

create index chromium_datasets_name_idx on chromium_datasets (name);
create index chromium_datasets_project_id_idx on chromium_datasets (project_id);
create index chromium_datasets_delivered_at_idx on chromium_datasets (delivered_at);

create index chromium_dataset_metrics_files_dataset_id_idx on chromium_dataset_metrics_files (dataset_id);
create index chromium_dataset_metrics_files_directory_idx on chromium_dataset_metrics_files (directory);
create index chromium_dataset_metrics_files_filename_idx on chromium_dataset_metrics_files (filename);
create index chromium_dataset_metrics_files_content_type_idx on chromium_dataset_metrics_files (content_type);

create index chromium_dataset_web_summaries_dataset_id_idx on chromium_dataset_web_summaries (dataset_id);
create index chromium_dataset_web_summaries_directory_idx on chromium_dataset_web_summaries (directory);
create index chromium_dataset_web_summaries_filename_idx on chromium_dataset_web_summaries (filename);
