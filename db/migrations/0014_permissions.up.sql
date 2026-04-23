-- 'auth_user' only needs to create and read people
grant insert, select on person, person_account to auth;

-- 'app_user' only needs to manage API keys
grant all on api_key to app;
