create table chromium_dataset (
    id uuid primary key default uuidv7(),
    name case_insensitive_text not null,
    delivered_at timestamptz not null
);

-- We don't actually store the content of the files in the database, just the path, so we can do permissions checks.
-- The actual files are stored on the server and served statically by caddy :)
create table chromium_dataset_raw_file (
    dataset_id uuid references chromium_dataset on delete cascade not null,
    path case_insensitive_text not null,
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
    dataset_id uuid references chromium_dataset on delete cascade not null,
    library_id uuid references library not null,
    primary key (dataset_id, library_id)
);

create function check_chromium_dataset_delivered_after_libraries_prepared() returns trigger language plpgsql volatile strict as $$
    declare
        library_prepared_at timestamptz;
        dataset_delivered_at timestamptz;
    begin
        select prepared_at from library where id = new.library_id into library_prepared_at;
        select delivered_at from chromium_dataset where id = new.dataset_id into dataset_delivered_at;

        if (library_prepared_at > dataset_delivered_at) then
            raise check_violation using message = 'Chromium dataset cannot be delivered before its constituent libraries were created', table = tg_table_name;
        end if;

        return new;
    end;
$$;

create trigger chromium_datasets_delivered_after_libraries_prepared before insert or update on chromium_dataset_library for each row execute function check_chromium_dataset_delivered_after_libraries_prepared();
