alter table suspension_pools add column multiplexing_type text;

update suspension_pools set multiplexing_type = 'exogenous_tag';

alter table suspension_pools alter column multiplexing_type set not null;

create table untagged_suspension_pooling (
    suspension_id uuid references suspensions on delete restrict on update restrict not null,
    pool_id uuid references suspension_pools on delete restrict on update restrict not null,

    primary key (suspension_id, pool_id)
);
