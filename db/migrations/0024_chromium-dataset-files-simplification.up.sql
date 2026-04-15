alter table chromium_dataset_files rename to chromium_dataset_raw_files;

create table chromium_dataset_parsed_files (
    dataset_id uuid references chromium_datasets on delete restrict on update restrict not null,
    path case_insensitive_text not null,
    data jsonb not null,
    primary key (dataset_id, path),
    foreign key (dataset_id, path) references chromium_dataset_raw_files
);

insert into chromium_dataset_parsed_files (dataset_id, path, data)
select
    dataset_id,
    path,
    parsed_data as data
from chromium_dataset_raw_files
where parsed_data is not null;

alter table chromium_dataset_raw_files add column content_encoding text;
alter table chromium_dataset_raw_files drop column parsed_data;

alter table chromium_datasets rename column file_links to raw_file_links;
alter table chromium_datasets add column parsed_file_links text [] default '{}' not null;

create function update_chromium_dataset_raw_file_links() returns trigger language plpgsql volatile strict as $$
    begin
        update chromium_datasets set raw_file_links = array(select distinct unnest(raw_file_links || ('/chromium-datasets/' ||
            id || '/raw-files/' || new.path)) order by 1) where id = new.dataset_id;
        return new;
    end;
$$;

create trigger append_raw_file_link after insert on chromium_dataset_raw_files for each row execute function
update_chromium_dataset_raw_file_links();

create function update_chromium_dataset_parsed_file_links() returns trigger language plpgsql volatile strict as $$
    begin
        update chromium_datasets set parsed_file_links = array(select distinct unnest(parsed_file_links || ('/chromium-datasets/' ||
            id || '/parsed-files/' || new.path)) order by 1) where id = new.dataset_id;
        return new;
    end;
$$;

create trigger append_file_link after insert on chromium_dataset_parsed_files for each row execute function
update_chromium_dataset_parsed_file_links();

drop function update_chromium_dataset_file_links cascade;
