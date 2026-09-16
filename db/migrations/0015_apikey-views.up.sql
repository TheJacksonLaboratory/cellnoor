create view api_key_public with (security_invoker = true) as (
    select
        api_key.id,
        api_key.description,
        api_key.owner_id,
        principal.is_staff as owner_is_staff,
        api_key.created_at,
        api_key.expires_at
    from api_key join principal on api_key.owner_id = principal.id
);

-- The application authenticates an API key before it knows who the user is, so this runs as the function's owner and
-- bypasses row-level security. It is the only way to find an API key by its hash
create function api_key_by_hash(
    hashed_key_ bytea
) returns setof api_key_public language sql stable security definer as $$
    select api_key_public from api_key_public where id = (select id from api_key where hashed_key = hashed_key_)
$$;
