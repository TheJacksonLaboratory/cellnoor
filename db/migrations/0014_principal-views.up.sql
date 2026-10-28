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

create view bot_account_public with (security_invoker = true) as (
    select
        bot_account.id,
        bot_account.description,
        bot_account.owned_by,
        principal.is_staff,
        bot_account.created_at
    from bot_account join principal on bot_account.id = principal.id
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

create view bot_account_detailed with (security_invoker = true) as (
    select
        bot_account_public,
        array(
            select (perm.resource, perm.action)::granted_permission from permission as perm
            where perm.principal_id = bot_account_public.id
        ) as permissions,
        array(
            select access.person_id from bot_account_access as access
            where access.bot_account_id = bot_account_public.id
        ) as members
    from bot_account_public
);
