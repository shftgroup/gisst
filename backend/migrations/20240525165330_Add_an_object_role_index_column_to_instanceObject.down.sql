-- Add down migration script here

ALTER TABLE instanceObject
DROP COLUMN object_role_index;
