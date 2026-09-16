-- Row-level security decides what 'app' may do with each row, so it gets every privilege on every table
grant select, insert, update, delete on all tables in schema public to app;
alter default privileges in schema public grant select, insert, update, delete on tables to app;

-- The exception is api_key: a hashed key must never leave the database, so 'app' cannot read that column. Looking a
-- key up by its hash goes through `api_key_by_hash` instead
revoke all on api_key from app;
grant insert (description, hashed_key, owner_id, expires_at),
select (id, description, owner_id, created_at, expires_at),
update (description, expires_at),
delete on api_key to app;
