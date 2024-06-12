-- Add up migration script here
ALTER TABLE instanceObject
ADD COLUMN object_role_index integer NOT NULL DEFAULT 0;
