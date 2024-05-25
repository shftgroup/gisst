-- Add down migration script here
ALTER TABLE instance
DROP COLUMN derived_from_instance,
DROP COLUMN derived_from_state;
