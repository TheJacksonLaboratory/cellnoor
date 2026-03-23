create table chromium_dataset_files (
    dataset_id uuid references chromium_datasets on delete restrict on update restrict not null,
    path case_insensitive_text not null,
    content_type case_insensitive_text not null,
    raw_content bytea not null,
    parsed_data jsonb,
    primary key (dataset_id, path)
);

create function update_chromium_dataset_file_links() returns trigger language plpgsql volatile strict as $$
    begin
        update chromium_datasets set file_links = array(select distinct unnest(file_links || ('/chromium-datasets/' ||
            id || '/files/' || new.path)) order by 1) where id = new.dataset_id;
        return new;
    end;
$$;

create trigger append_file_link after insert on chromium_dataset_files for each row execute function
update_chromium_dataset_file_links();

drop function initialize_chromium_dataset_links cascade;
drop function update_web_summaries_links cascade;
drop function update_metrics_files_links cascade;

-- Files with paths like "cellranger/web_summary.html" should just be flattened to "web_summary.html"
insert into chromium_dataset_files (dataset_id, path, content_type, raw_content, parsed_data)
select
    dataset_id,
    filename as path,
    content_type,
    raw_content,
    parsed_data
from chromium_dataset_metrics_files
where directory like '%cellranger%';

insert into chromium_dataset_files (dataset_id, path, content_type, raw_content)
select
    dataset_id,
    filename as path,
    'text/html' as content_type,
    content as raw_content
from chromium_dataset_web_summaries
where directory like '%cellranger%';


-- Files with paths like "sample_name/web_summary.html" should be nested into
-- "per_sample_outs/sample_name/web_summary.html" to better mirror cellranger-multi's outputs
insert into chromium_dataset_files (dataset_id, path, content_type, raw_content, parsed_data)
select
    dataset_id,
    'per_sample_outs/' || directory || '/' || filename as path,
    content_type,
    raw_content,
    parsed_data
from chromium_dataset_metrics_files
where directory not like '%cellranger%';

insert into chromium_dataset_files (dataset_id, path, content_type, raw_content)
select
    dataset_id,
    'per_sample_outs/' || directory || '/' || filename as path,
    'text/html' as content_type,
    content as raw_content
from chromium_dataset_web_summaries
where directory not like '%cellranger%';


drop table chromium_dataset_metrics_files;
drop table chromium_dataset_web_summaries;
