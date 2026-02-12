-- Your SQL goes here
alter table specimen_measurements add unique (specimen_id, measured_by, measured_at, data);

alter table suspension_pool_measurements add unique (pool_id, measured_by, measured_at, data);

alter table suspension_measurements add unique (suspension_id, measured_by, measured_at, data);

alter table cdna_measurements add unique (cdna_id, measured_by, measured_at, data);

alter table library_measurements add unique (library_id, measured_by, measured_at, data);
