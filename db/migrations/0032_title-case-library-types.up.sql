update cdna set library_type = 'Antibody Capture'
where library_type = 'antibody_capture';
update cdna set library_type = 'Antigen Capture'
where library_type = 'antigen_capture';
update cdna set library_type = 'Chromatin Accessibility'
where library_type = 'chromatin_accessibility';
update cdna set library_type = 'CRISPR Guide Capture'
where library_type = 'crispr_guide_capture';
update cdna set library_type = 'Custom'
where library_type = 'custom';
update cdna set library_type = 'Gene Expression'
where library_type = 'gene_expression';
update cdna set library_type = 'Multiplexing Capture'
where library_type = 'multiplexing_capture';
update cdna set library_type = 'VDJ'
where library_type = 'vdj';
update cdna set library_type = 'VDJ B'
where library_type = 'vdj_b';
update cdna set library_type = 'VDJ T'
where library_type = 'vdj_t';
update cdna set library_type = 'VDJ T GD'
where library_type = 'vdj_t_gd';

update library_type_specification set library_type = 'Antibody Capture'
where library_type = 'antibody_capture';
update library_type_specification set library_type = 'Antigen Capture'
where library_type = 'antigen_capture';
update library_type_specification set library_type = 'Chromatin Accessibility'
where library_type = 'chromatin_accessibility';
update library_type_specification set library_type = 'CRISPR Guide Capture'
where library_type = 'crispr_guide_capture';
update library_type_specification set library_type = 'Custom'
where library_type = 'custom';
update library_type_specification set library_type = 'Gene Expression'
where library_type = 'gene_expression';
update library_type_specification set library_type = 'Multiplexing Capture'
where library_type = 'multiplexing_capture';
update library_type_specification set library_type = 'VDJ'
where library_type = 'vdj';
update library_type_specification set library_type = 'VDJ B'
where library_type = 'vdj_b';
update library_type_specification set library_type = 'VDJ T'
where library_type = 'vdj_t';
update library_type_specification set library_type = 'VDJ T GD'
where library_type = 'vdj_t_gd';

update tenx_assay set library_types = array_replace(library_types, 'antibody_capture', 'Antibody Capture');
update tenx_assay set library_types = array_replace(library_types, 'antigen_capture', 'Antigen Capture');
update tenx_assay set
    library_types = array_replace(library_types, 'chromatin_accessibility', 'Chromatin Accessibility');
update tenx_assay set library_types = array_replace(library_types, 'crispr_guide_capture', 'CRISPR Guide Capture');
update tenx_assay set library_types = array_replace(library_types, 'custom', 'Custom');
update tenx_assay set library_types = array_replace(library_types, 'gene_expression', 'Gene Expression');
update tenx_assay set library_types = array_replace(library_types, 'multiplexing_capture', 'Multiplexing Capture');
update tenx_assay set library_types = array_replace(library_types, 'vdj', 'VDJ');
update tenx_assay set library_types = array_replace(library_types, 'vdj_b', 'VDJ B');
update tenx_assay set library_types = array_replace(library_types, 'vdj_t', 'VDJ T');
update tenx_assay set library_types = array_replace(library_types, 'vdj_t_gd', 'VDJ T GD');

update chromium_dataset_parsed_file set
    data = jsonb_build_object('format', 'standard_csv', 'data', jsonb_build_array(data -> 'data'))
where data ->> 'format' = 'single_row_csv';

update chromium_dataset_parsed_file set data = jsonb_set(data, '{format}', '"transposed_csv"')
where data ->> 'format' = 'multi_row_csv' and data -> 'data' -> 0 ? 'Metric Name';

update chromium_dataset_parsed_file set data = jsonb_set(data, '{format}', '"standard_csv"')
where data ->> 'format' = 'multi_row_csv';
