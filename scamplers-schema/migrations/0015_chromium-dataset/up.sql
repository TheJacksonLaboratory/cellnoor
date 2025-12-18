create table chromium_datasets (
    id uuid primary key default uuidv7(),
    links jsonb default '{}' not null,
    name case_insensitive_text not null,
    lab_id uuid references labs on delete restrict on update restrict not null,
    delivered_at timestamptz not null
);

create table chromium_dataset_libraries (
    dataset_id uuid references chromium_datasets on delete restrict on update restrict not null,
    library_id uuid references libraries on delete restrict on update restrict not null,
    primary key (dataset_id, library_id)
);

create table chromium_dataset_metrics_files (
    dataset_id uuid references chromium_datasets on delete restrict on update restrict not null,
    directory case_insensitive_text not null,
    filename case_insensitive_text not null,
    content_type case_insensitive_text not null,
    raw_content bytea not null,
    parsed_data jsonb not null,
    primary key (dataset_id, directory, filename)
);

create table chromium_dataset_web_summaries (
    dataset_id uuid references chromium_datasets on delete restrict on update restrict not null,
    directory case_insensitive_text not null,
    filename case_insensitive_text not null,
    content bytea not null,
    primary key (dataset_id, directory, filename)
);

create function initialize_chromium_dataset_links() returns trigger language plpgsql volatile strict as $$
    begin
        new.links = json_object(
            'self_': '/chromium-datasets/' || new.id,
            'specimens': '/chromium-datasets/' || new.id || '/specimens',
            'libraries': '/chromium-datasets/' || new.id || '/libraries',
            'web-summaries': jsonb_build_array(),
            'metrics-files': jsonb_build_array()
        );
        return new;
    end;
$$;

create function update_web_summaries_links() returns trigger language plpgsql volatile strict as $$
    begin
        update chromium_datasets set links = jsonb_set(links, '{web-summaries}', links -> 'web-summaries' || jsonb_build_array('/chromium-datasets/' || id || '/web-summaries/' || new.directory || '/' || new.filename)) where id = new.dataset_id;
        return new;
    end;
$$;

create function update_metrics_files_links() returns trigger language plpgsql volatile strict as $$
    begin
        update chromium_datasets set links = jsonb_set(links, '{metrics-files}', links -> 'metrics-files' || jsonb_build_array('/chromium-datasets/' || id || '/metrics-files/' || new.directory || '/' || new.filename)) where id = new.dataset_id;
        return new;
    end;
$$;

create trigger insert_links before insert on chromium_datasets for each row execute function
initialize_chromium_dataset_links();

create trigger append_web_summary_link after insert on chromium_dataset_web_summaries for each row execute function
update_web_summaries_links();

create trigger append_metrics_file_link after insert on chromium_dataset_metrics_files for each row execute function
update_metrics_files_links();
