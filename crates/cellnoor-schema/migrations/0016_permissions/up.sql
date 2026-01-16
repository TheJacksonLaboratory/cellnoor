-- cellnoor-api is the main app user. It can do anything, but it's gated behind authentication
grant all on all tables in schema public to cellnoor_api;

-- cellnoor_ui creates and updates people, but it can't do anything else (I don't trust my JavaScript skills). It's
-- also gated behind authentication
grant select, insert, update on people to cellnoor_ui;

grant insert, select, delete on json_web_keys, json_web_tokens to cellnoor_ui;

-- Users should be able to revoke tokens from the UI too, and it will be quicker if the UI does it instead of sending
-- a request to the API. This is fine because the API route that revokes tokens is unauthenticated anyways (so that
-- users can help one another in case of a leaked token. Inspired by GitHub's system)
grant insert, delete on revoked_json_web_tokens to cellnoor_ui;
