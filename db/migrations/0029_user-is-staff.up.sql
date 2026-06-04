create or replace function current_user_is_staff() returns boolean language plpgsql volatile strict as $$
    declare
        user_is_staff boolean;
        current_user_id uuid = current_user::uuid;
    begin
        -- The simple case: it's just a person
        select is_staff
        from person_public
        where id = current_user_id into user_is_staff;

        if user_is_staff then
            return user_is_staff;
        end if;

        -- The next case: it's a person using an API key
        select pers.is_staff
        from api_key as ak
        join person_public as pers on ak.person_id = pers.id
        where ak.id = current_user_id
        into user_is_staff;

        if user_is_staff then
            return user_is_staff;
        end if;

        -- Finally: it's a service account, which should have `is_staff == all(person.is_staff)` for every person who can use the service account
        select all (pers.is_staff) and svc_owner.is_staff
        from service_account_access as svc_acc
        join service_account as svc on svc_acc.service_account_id = svc.id
        join person_public as pers on svc_acc.person_id = pers.id
        join person_public as svc_owner on svc.owned_by = svc_owner.id
        where svc.id = current_user_id
        into user_is_staff;

        return user_is_staff;
    end;
$$;
