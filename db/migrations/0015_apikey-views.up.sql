create view api_key_public with (security_invoker = true) as (
    select
        id,
        description,
        person_id,
        service_account_id,
        created_at,
        expires_at
    from api_key
);
