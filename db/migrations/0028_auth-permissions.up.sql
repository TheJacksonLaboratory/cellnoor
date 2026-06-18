-- 'auth' only needs to create and read people
grant insert, select, update on person, account to auth;
grant select on institution to auth;

alter user auth with createrole bypassrls;
