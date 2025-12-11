create table chromium_datasets (
    id uuid primary key default uuidv7(),
    links jsonb generated always as (
        construct_links('chromium-datasets', id, '{"specimens", "libraries"}')
    ) stored not null,
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


create function insert_empty_web_summary_links() returns trigger language plpgsql volatile strict as $$
    begin
        new.links = new.links || '{"web-summaries": []}';
        return new;
    end;
$$;

create function update_web_summary_links() returns trigger language plpgsql volatile strict as $$
    begin
        update chromium_dataset set links = jsonb_set(links, '{"web-summaries"}', links -> 'web-summaries' || ('/chromium-datasets/' || id || '/web-summaries/' || new.filename));
        return new;
    end;
$$;

create trigger insert_empty_links before insert on chromium_datasets for each row execute function
insert_empty_web_summary_links();

create trigger append_link before insert on chromium_dataset_web_summaries for each row execute function
update_web_summary_links();
