alter user app with bypassrls;

-- 'app' needs to be able to read API keys directly to authenticate people
grant select on api_key to app;
