create table chromium_dataset (
    id uuid primary key default uuidv7(),
    name case_insensitive_text not null,
    delivered_at timestamptz not null
);

-- These are more complicated to compute, so we compute them on-demand in the view rather than storing them in the table
create type chromium_dataset_links as (
    self text,
    raw_files text[],
    parsed_files text[]
);

-- We don't actually store the content of the files in the database, just the path, so we can do permissions checks.
-- The actual files are stored on the server and served statically by caddy :)
create table chromium_dataset_raw_file (
    dataset_id uuid references chromium_dataset on delete cascade not null,
    path case_insensitive_text not null,
    primary key (dataset_id, path)
);

create function update_chromium_dataset_raw_file_links() returns trigger language plpgsql volatile strict as $$
    begin
        update chromium_dataset set links.self = id, links.raw_files = array(select distinct unnest(links.raw_files || ('/chromium-datasets/' ||
            id || '/raw-files/' || new.path)) order by 1) where id = new.dataset_id;
        return new;
    end;
$$;

create trigger append_raw_file_link after insert on chromium_dataset_raw_file for each row execute function
update_chromium_dataset_raw_file_links();

-- Some files can be parsed into JSON, so we store those
create table chromium_dataset_parsed_file (
    dataset_id uuid not null,
    path case_insensitive_text not null,
    data jsonb not null,
    primary key (dataset_id, path),
    foreign key (dataset_id, path) references chromium_dataset_raw_file on delete cascade
);

create function update_chromium_dataset_parsed_file_links() returns trigger language plpgsql volatile strict as $$
    begin
        update chromium_dataset set links.parsed_files = array(select distinct unnest(links.parsed_files || ('/chromium-datasets/' ||
            id || '/parsed-files/' || new.path)) order by 1) where id = new.dataset_id;
        return new;
    end;
$$;

create trigger append_file_link after insert on chromium_dataset_parsed_file for each row execute function
update_chromium_dataset_parsed_file_links();

create table chromium_dataset_library (
    dataset_id uuid references chromium_dataset on delete cascade not null,
    library_id uuid references library not null,
    primary key (dataset_id, library_id)
);
