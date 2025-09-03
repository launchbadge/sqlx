-- Add up migration script here
-- enable-substitution
CREATE USER ${my_user} WITH ENCRYPTED PASSWORD '${my_password}' INHERIT;
-- disable-substitution
