create table labs (
    id uuid primary key default uuidv7(),
    links jsonb generated always as (
        construct_links('labs', id, '{"members", "specimens", "chromium-datasets"}')
    ) stored not null,
    name case_insensitive_text unique not null,
    pi_id uuid references people on delete restrict on update restrict not null,
    delivery_dir case_insensitive_text unique not null
);

create table lab_membership (
    lab_id uuid references labs on delete restrict on update restrict not null,
    member_id uuid references people on delete restrict on update restrict not null,
    primary key (lab_id, member_id)
);
