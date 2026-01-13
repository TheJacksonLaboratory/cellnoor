-- Anyone can read any table for now
grant select on all tables in schema public to public;

-- Users with app_admin can do anything (TODO: app_admin should be more restricted)
grant all on all tables in schema public to app_admin;

-- cellnoor_ui creates people and service accounts
grant insert on people, service_accounts to cellnoor_ui;
grant update on people to cellnoor_ui;

-- Anyone can delete a service account
grant delete on service_accounts to public;

-- A person can only create service accounts for themselves, but cellnoor_api needs to see all of them and cellnoor_ui
-- needs to create them
alter table service_accounts enable row level security;
create policy user_service_account on service_accounts using (
    current_user = created_by::text or current_user in ('cellnoor_api', 'cellnoor_ui')
);
