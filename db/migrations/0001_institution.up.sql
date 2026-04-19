create type institution_links as (
    self text
);

create table institution (
    id uuid primary key,
    links institution_links generated always as (row('/institutions/' || id)) stored not null,
    name case_insensitive_text unique not null
);
