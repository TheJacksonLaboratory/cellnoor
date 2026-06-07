create view person_public as (
    select
        id,
        name,
        email,
        institution_id,
        is_staff,
        can_admin_users,
        orcid
    from person
);
