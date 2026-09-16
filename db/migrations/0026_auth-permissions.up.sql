-- 'auth' only needs to create and read people, and nobody is logged in yet when it does, so it bypasses row-level
-- security. Inserting a person also inserts their principal
grant select on principal to auth;
grant insert, select, update on person, account to auth;
grant select on institution to auth;

alter user auth with bypassrls;
