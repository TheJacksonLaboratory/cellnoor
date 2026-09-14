create table chromium_dataset (
    id uuid primary key default uuidv7(),
    name case_insensitive_text not null,
    delivered_at timestamptz not null,

    unique (id, delivered_at)
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
