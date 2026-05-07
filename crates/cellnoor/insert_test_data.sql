insert into project (id, name, started_at, ended_at)
values (uuid_nil(), 'project0', '1999-01-01T00:00:00Z', '1999-01-01T00:00:0Z');

insert into tenx_assay (id, name, chemistry_version, protocol_url) values (
    uuid_nil(), 'singleplex_gene_expression', '', ''
);

insert into index_kit (name) values ('index_kit');

insert into single_index_set (name, kit, well, sequences)
values ('single_index_set', 'index_kit', 'well', array['']);

insert into specimen (
    id,
    readable_id,
    name,
    submitted_by,
    project_id,
    received_at,
    species,
    type,
    tissue
)
values (
    uuid_nil(),
    'specimen0',
    'singleplex_gene_expression_specimen',
    uuid_nil(),
    uuid_nil(),
    '1999-01-01T00:00:00Z',
    'species0',
    'type0',
    'tissue0'
);

insert into suspension (id, readable_id, specimen_id, content)
values (uuid_nil(), 'suspension0', uuid_nil(), 'content0');

insert into chromium_run (id, readable_id, assay_id, run_at, run_by, succeeded)
values (
    uuid_nil(),
    'chromium_run0',
    uuid_nil(),
    '1999-01-01T00:00:00Z',
    uuid_nil(),
    true
);

insert into gem_pool (id, readable_id, chromium_run_id)
values (uuid_nil(), 'gem_pool0', uuid_nil());

insert into chip_loading (
    id,
    gem_pool_id,
    suspension_id,
    suspension_volume_loaded,
    buffer_volume_loaded
)
values (uuid_nil(), uuid_nil(), uuid_nil(), '{}'::jsonb, '{}'::jsonb);

insert into cdna (
    id,
    readable_id,
    library_type,
    prepared_at,
    gem_pool_id,
    n_amplification_cycles
)
values (
    uuid_nil(),
    'cdna0',
    'Gene Expression',
    '1999-01-01T00:00:00Z',
    uuid_nil(),
    0
);

insert into library (
    id,
    readable_id,
    cdna_id,
    single_index_set_name,
    number_of_sample_index_pcr_cycles,
    prepared_at
)
values (
    uuid_nil(),
    'library0',
    uuid_nil(),
    'single_index_set',
    0,
    '1999-01-01T00:00:00Z'
);

insert into chromium_dataset (id, name, delivered_at)
values (
    uuid_nil(),
    'singleplex_gene_expression_chromium_dataset',
    '1999-01-01T00:00:00Z'
);

insert into chromium_dataset_library (dataset_id, library_id)
values (uuid_nil(), uuid_nil());
