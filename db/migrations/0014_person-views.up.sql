create view person_public as (
    select
        id,
        name,
        email,
        institution_id,
        is_staff,
        can_manage_users,
        orcid
    from person
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
