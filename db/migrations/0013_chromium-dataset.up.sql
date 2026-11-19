create table reference_package (
    package_name case_insensitive_text primary key,
    genome_assembly case_insensitive_text not null,
    tenx_reference_version case_insensitive_text,
    assembly_source case_insensitive_text not null,
    assembly_release case_insensitive_text not null,
    annotation_source case_insensitive_text not null,
    annotation_release case_insensitive_text not null
);

insert into reference_package (
    package_name,
    genome_assembly,
    tenx_reference_version,
    assembly_source,
    assembly_release,
    annotation_source,
    annotation_release
) values
-- https://www.10xgenomics.com/support/software/cell-ranger/latest/release-notes/cr-reference-release-notes
('refdata-gex-GRCh38-2024-A', 'GRCh38', '2024-A', 'Ensembl', '109', 'GENCODE', 'v44'),
('refdata-gex-GRCm39-2024-A', 'GRCm39', '2024-A', 'Ensembl', '110', 'GENCODE', 'vM33'),
('refdata-gex-mRatBN7-2-2024-A', 'mRatBN7.2', '2024-A', 'Ensembl', '110', 'Ensembl', '110'),
(
    'refdata-gex-GRCh38_and_GRCm39-2024-A',
    'GRCh38_and_GRCm39',
    '2024-A',
    'Ensembl',
    '109_and_110',
    'GENCODE',
    'v44_and_vM33'
),

-- https://www.10xgenomics.com/support/software/cell-ranger/downloads/prebuilt-vdj-references
('refdata-cellranger-vdj-GRCh38-alts-ensembl-7.1.0', 'GRCh38', 'alts-ensembl-7.1.0', 'Ensembl', '94', 'Ensembl', '94'),
('refdata-cellranger-vdj-GRCm38-alts-ensembl-7.0.0', 'GRCm38', 'alts-ensembl-7.0.0', 'Ensembl', '94', 'Ensembl', '94'),

-- https://www.10xgenomics.com/support/software/cell-ranger-arc/downloads#reference-downloads and
-- https://www.10xgenomics.com/support/software/cell-ranger-arc/downloads/ref-build-steps#ref-2024-a
('refdata-cellranger-arc-GRCh38-2024-A', 'GRCh38', '2024-A', 'Ensembl', '109', 'GENCODE', 'v44'),
('refdata-cellranger-arc-GRCm39-2024-A', 'GRCm39', '2024-A', 'Ensembl', '110', 'GENCODE', 'vM33'),

-- https://www.10xgenomics.com/support/software/cell-ranger-arc/latest/release-notes/reference-release-notes and
-- https://www.10xgenomics.com/support/software/cell-ranger-arc/downloads/ref-build-steps#ref-2020-a
('refdata-cellranger-arc-GRCh38-2020-A-2.0.0', 'GRCh38', '2020-A-2.0.0', 'Ensembl', '98', 'GENCODE', 'v32'),
('refdata-cellranger-arc-mm10-2020-A-2.0.0', 'mm10', '2020-A-2.0.0', 'Ensembl', '98', 'GENCODE', 'vM23');

create table chromium_data_processing_pipeline (
    pipeline_name case_insensitive_text not null,
    command case_insensitive_text not null,
    version case_insensitive_text not null,

    primary key (pipeline_name, command, version)
);

insert into chromium_data_processing_pipeline (pipeline_name, command, version)
select
    'cellranger',
    command,
    version
from
    unnest('{count, multi, vdj}'::text[]) as command,
    unnest('{7.0.0, 7.1.0, 7.2.0, 8.0.0, 8.0.1, 9.0.0, 9.0.1, 10.0.0, 10.1.0}'::text[]) as version
union all
select
    'cellranger-arc',
    'count',
    unnest('{2.0.2, 2.1.0, 2.2.0}'::text[])
union all
select
    'cellranger-atac',
    'count',
    unnest('{2.1.0, 2.2.0}'::text[]);

create table chromium_dataset (
    id uuid primary key default uuidv7(),
    name case_insensitive_text not null,
    delivered_at timestamptz not null,
    reference_package_name case_insensitive_text references reference_package not null,
    pipeline_name case_insensitive_text not null,
    pipeline_command case_insensitive_text not null,
    pipeline_version case_insensitive_text not null,
    additional_data jsonb,

    unique (id, delivered_at),
    foreign key (pipeline_name, pipeline_command, pipeline_version) references chromium_data_processing_pipeline
);

-- We don't actually store the content of the files in the database, just the path, so we can do permissions checks.
-- The actual files are stored on the server and served statically by caddy :)
create table chromium_dataset_raw_file (
    dataset_id uuid references chromium_dataset on delete cascade not null,
    path case_insensitive_text not null,
    sha256_hash bytea not null,
    primary key (dataset_id, path)
);

-- Some files can be parsed into JSON, so we store those
create table chromium_dataset_parsed_file (
    dataset_id uuid not null,
    path case_insensitive_text not null,
    data jsonb not null,
    primary key (dataset_id, path),
    foreign key (dataset_id, path) references chromium_dataset_raw_file on delete cascade
);

create table chromium_dataset_library (
    dataset_id uuid not null,
    dataset_delivered_at timestamptz not null,
    library_id uuid not null,
    library_prepared_at timestamptz not null,
    primary key (dataset_id, library_id),

    foreign key (dataset_id, dataset_delivered_at) references chromium_dataset (
        id, delivered_at
    ) on update cascade on delete cascade,
    foreign key (library_id, library_prepared_at) references library (id, prepared_at) on update cascade,

    constraint delivered_after_library_prepared check (dataset_delivered_at >= library_prepared_at)
);

create function populate_chromium_dataset_library_timestamps() returns trigger language plpgsql as $$
    begin
        select delivered_at into new.dataset_delivered_at from chromium_dataset where id = new.dataset_id;
        select prepared_at into new.library_prepared_at from library where id = new.library_id;

        return new;
    end;
$$;

create trigger chromium_dataset_library_timestamps
before insert or update of dataset_id, library_id on chromium_dataset_library
for each row execute function populate_chromium_dataset_library_timestamps();
