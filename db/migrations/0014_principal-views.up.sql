-- `security_invoker = true` means that the query's security checks run as the "invoker" (the application user), not
-- the owner of the view, so row-level security applies
create view person_public with (security_invoker = true) as (
    select
        id,
        name,
        email,
        institution_id,
        is_staff,
        orcid
    from person
);

create view person_account with (security_invoker = true) as (
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

-- A permission without the principal it belongs to, so that it can be aggregated per principal
create type granted_permission as (
    resource case_insensitive_text,
    action case_insensitive_text
);

create view person_with_permissions with (security_invoker = true) as (
    select
        person_public,
        array(
            select (perm.resource, perm.action)::granted_permission from permission as perm
            where perm.principal_id = person_public.id
        ) as permissions
    from person_public
);

create view service_detailed with (security_invoker = true) as (
    select
        service_public,
        array(
            select (perm.resource, perm.action)::granted_permission from permission as perm
            where perm.principal_id = service_public.id
        ) as permissions,
        array(
            select access.person_id from service_access as access
            where access.service_id = service_public.id
        ) as members
    from service_public
);
