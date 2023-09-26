-- Add down migration script here
ALTER TABLE IF EXISTS creator ALTER COLUMN creator_username SET DATA TYPE varchar(20);
ALTER TABLE IF EXISTS creator ALTER COLUMN creator_full_name SET DATA TYPE varchar(50);
