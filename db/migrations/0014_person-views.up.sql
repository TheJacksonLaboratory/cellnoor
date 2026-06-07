create view person_public as (
    select
        id,
        name,
        email,
        institution_id,
        can_read_all_projects,
        can_admin_users,
        orcid
    from person
);
