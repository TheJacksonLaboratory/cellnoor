-- cellnoor-api is the main app user, so it needs to do basically anything the app supports
grant all on all tables in schema public to cellnoor_api;

-- cellnoor-api shouldn't be able to modify `json_web_keys` or read the private key (even though it's encrypted)
revoke all on json_web_keys from cellnoor_api;
grant select (id, public_key, expires_at) on json_web_keys to cellnoor_api;

-- cellnoor_ui creates and updates people, but it can't do much else (I don't trust my JavaScript skills). It also
-- needs to read users' labs when issuing tokens
grant select, insert, update on people to cellnoor_ui;
grant select (id, pi_id) on labs to cellnoor_ui;
grant select (lab_id, member_id) on lab_membership to cellnoor_ui;

-- JWKs and JWTs are managed by better-auth, so cellnoor-ui needs permissions on those tables
grant insert, select, delete on json_web_keys, json_web_tokens to cellnoor_ui;

-- Users should be able to revoke tokens from the UI too, and it will be quicker if the UI does it instead of sending
-- a request to the API. This is fine because the API route that revokes tokens is unauthenticated anyways (so that
-- users can help one another in case of a leaked token. Inspired by GitHub's system)
grant insert, delete on revoked_json_web_tokens to cellnoor_ui;
