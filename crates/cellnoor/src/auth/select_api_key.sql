select (id, description, person_id, service_account_id, created_at, expires_at)::api_key_public from api_key
where hashed_key = $1;
