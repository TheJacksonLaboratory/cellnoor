-- Test data for Chromium experiments.

-- Variant 1: one specimen -> one suspension -> one gem_pool (no OCM barcode) in one chromium_run
--            -> one cDNA -> one library -> one chromium_dataset

insert into project (id, name, started_at, ended_at)
values (uuid_nil(), '', '1999-01-01T00:00:00Z', '1999-01-01T00:00:0Z');

insert into specimen (id, readable_id, name, submitted_by, project_id, received_at, species, type, tissue)
values (
    uuid_nil(),
    'singleplex_gene_expression',
    '',
    uuid_nil(),
    uuid_nil(),
    '1999-01-01T00:00:00Z',
    '',
    '',
    ''
);

insert into suspension (id, readable_id, specimen_id, content)
values (uuid_nil(), 'singleplex_gene_expression', uuid_nil(), '');

insert into tenx_assay (id, name, chemistry_version, protocol_url)
values (uuid_nil(), '', '', '');

insert into chromium_run (id, readable_id, assay_id, run_at, run_by, succeeded)
values (uuid_nil(), 'singleplex_gene_expression', uuid_nil(), '1999-01-01T00:00:00Z', uuid_nil(), true);

insert into gem_pool (id, readable_id, chromium_run_id)
values (uuid_nil(), 'singleplex_gene_expression', uuid_nil());

insert into chip_loading (id, gem_pool_id, suspension_id, suspension_volume_loaded, buffer_volume_loaded)
values (uuid_nil(), uuid_nil(), uuid_nil(), '{}'::jsonb, '{}'::jsonb);

insert into cdna (id, readable_id, library_type, prepared_at, gem_pool_id, n_amplification_cycles)
values (uuid_nil(), 'singleplex_gene_expression', 'Gene Expression', '1999-01-01T00:00:00Z', uuid_nil(), 0);

insert into index_kit (name) values ('');

insert into single_index_set (name, kit, well, sequences)
values ('', '', '', array['']);

insert into library (
    id, readable_id, cdna_id, single_index_set_name, number_of_sample_index_pcr_cycles, prepared_at
)
values (uuid_nil(), 'singleplex_gene_expression', uuid_nil(), '', 0, '1999-01-01T00:00:00Z');

insert into chromium_dataset (id, name, delivered_at)
values (uuid_nil(), '', '1999-01-01T00:00:00Z');

insert into chromium_dataset_library (dataset_id, library_id)
values (uuid_nil(), uuid_nil());
