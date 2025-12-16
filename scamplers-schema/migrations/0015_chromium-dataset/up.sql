create table chromium_datasets (
    id uuid primary key default uuidv7(),
    links jsonb default '{}' not null,
    name case_insensitive_text not null,
    lab_id uuid references labs on delete restrict on update restrict not null,
    data_path case_insensitive_text not null,
    delivered_at timestamptz not null,
    parsed_metrics_files jsonb not null
);

create table chromium_dataset_libraries (
    dataset_id uuid references chromium_datasets on delete restrict on update restrict not null,
    library_id uuid references libraries on delete restrict on update restrict not null,
    primary key (dataset_id, library_id)
);

create table chromium_dataset_web_summaries (
    filename case_insensitive_text not null,
    dataset_id uuid references chromium_datasets on delete restrict on update restrict not null,
    content bytea not null,
    primary key (filename, dataset_id)
);


create function initialize_chromium_dataset_links() returns trigger language plpgsql volatile strict as $$
    begin
        new.links = json_object(
            'self_': '/chromium-datasets/' || new.id,
            'specimens': '/chromium-datasets/' || new.id || '/specimens',
            'libraries': '/chromium-datasets/' || new.id || '/libraries',
            'web-summaries': jsonb_build_array()
        );
        return new;
    end;
$$;

create function update_web_summary_links() returns trigger language plpgsql volatile strict as $$
    begin
        update chromium_datasets set links = jsonb_set(links, '{web-summaries}', links -> 'web-summaries' || jsonb_build_array('/chromium-datasets/' || id || '/web-summaries/' || new.filename)) where id = new.dataset_id;
        return new;
    end;
$$;

create trigger insert_links before insert on chromium_datasets for each row execute function
initialize_chromium_dataset_links();

create trigger append_link after insert on chromium_dataset_web_summaries for each row execute function
update_web_summary_links();
