select api_key_public from api_key_public
where id in (
    select id from api_key
    where hashed_key = $1
);
