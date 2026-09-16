-- `security_invoker = true` means that the query's security checks run as the "invoker" (the application user), not
-- the owner of the view, so row-level security applies
create view person_public with (security_invoker = true) as (
    select
        person.id,
        person.name,
        person.email,
        person.institution_id,
        principal.is_staff,
        person.orcid
    from person join principal on person.id = principal.id
);

create view person_account as (
    select
        person.id,
        person.name,
        person.email,
        account.auth_provider,
        account.auth_provider_user_id
    from account join person on account.person_id = person.id
);

create view service_public with (security_invoker = true) as (
    select
        service.id,
        service.description,
        service.owned_by,
        principal.is_staff,
        service.created_at
    from service join principal on service.id = principal.id
);
