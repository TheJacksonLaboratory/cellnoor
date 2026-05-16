-- We include the multiplexing tag because it's cheap and is useful for consumers
create type tagged_specimen as (
    specimen specimen,
    multiplexing_tag multiplexing_tag,
    ocm_barcode_id case_insensitive_text
);

create view suspension_pool_to_specimen as (
    select
        suspension_pool,
        suspension.specimen,
        suspension,
        multiplexing_tag
    from suspension_pool
    join suspension_pooling as pooling on suspension_pool.id = pooling.pool_id
    join suspension_to_specimen as suspension on pooling.suspension_id = (suspension.suspension).id
    left join multiplexing_tag on pooling.tag_id = multiplexing_tag.id
);

create function get_suspension_created_at(suspension_id uuid) returns timestamptz language plpgsql volatile strict as $$
    begin
        return (select greatest((suspension).created_at, (specimen).received_at) from suspension_to_specimen where (suspension).id = suspension_id);
    end;
$$;

create function check_suspensions_pooled_after_suspension_creation() returns trigger language plpgsql volatile strict as $$
    declare
        suspension_created_at timestamptz = get_suspension_created_at(new.suspension_id);
        suspension_pool_pooled_at timestamptz;
    begin
        select pooled_at from suspension_pool where id = new.pool_id into suspension_pool_pooled_at;

        if (suspension_created_at > suspension_pool_pooled_at) then
            raise check_violation using message = 'suspension pool cannot be created before its constituent suspensions', table = tg_table_name;
        end if;

        return new;
    end;
$$;

create trigger suspensions_pooled_after_creation before insert or update on suspension_pooling for each row execute function check_suspensions_pooled_after_suspension_creation();
