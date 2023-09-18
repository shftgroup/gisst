-- Add down migration script here
ALTER TABLE user DROP CONSTRAINT user_creator_id_derived_from_fkey;
DROP TABLE user;
DROP TYPE IF EXISTS auth_provider;