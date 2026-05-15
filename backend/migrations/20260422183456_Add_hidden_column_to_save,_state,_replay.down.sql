-- Add down migration script here
ALTER TABLE save DROP COLUMN hidden;
ALTER TABLE state DROP COLUMN hidden;
ALTER TABLE replay DROP COLUMN hidden;
