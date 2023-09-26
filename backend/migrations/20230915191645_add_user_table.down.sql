-- Add down migration script here
ALTER TABLE users DROP CONSTRAINT users_creator_id_fkey;
DROP TABLE users;
DROP TYPE IF EXISTS auth_provider;