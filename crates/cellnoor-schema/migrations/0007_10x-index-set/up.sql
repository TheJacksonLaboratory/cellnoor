-- We allow for text primary keys here because the names of these things will not change
create table index_kits (
    name case_insensitive_text primary key
);

create table single_index_sets (
    name case_insensitive_text primary key,
    kit case_insensitive_text references index_kits on delete restrict on update restrict not null,
    well case_insensitive_text not null,
    sequences case_insensitive_text [] not null
);

create table dual_index_sets (
    name case_insensitive_text primary key,
    kit case_insensitive_text references index_kits on delete restrict on update restrict not null,
    well case_insensitive_text not null,
    index_i7 case_insensitive_text not null,
    index2_workflow_a_i5 case_insensitive_text not null,
    index2_workflow_b_i5 case_insensitive_text not null
);
