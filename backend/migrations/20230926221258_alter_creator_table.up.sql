-- Add up migration script here

ALTER TABLE IF EXISTS creator ALTER COLUMN creator_username SET DATA TYPE text;
ALTER TABLE IF EXISTS creator ALTER COLUMN creator_full_name SET DATA TYPE text;
