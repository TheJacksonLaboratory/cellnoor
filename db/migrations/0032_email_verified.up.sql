-- better-auth only links a person to their account upon sign-in if:
-- 1. the email returned by the auth provider is the same as an email in this table
-- 2. email_verified == true
-- Thus, it's necessary to set email_verified = true so that people who were created with only an email can sign in
-- and have the correct permissions and projects
alter table person alter column email_verified set default true;

update person set email_verified = true;
