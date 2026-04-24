-- Add down migration script here
ALTER TABLE work DROP COLUMN creator_id;
ALTER TABLE environment DROP COLUMN creator_id;
ALTER TABLE instance DROP COLUMN creator_id;
ALTER TABLE object DROP COLUMN creator_id;
ALTER TABLE file DROP COLUMN creator_id;
ALTER TABLE screenshot DROP COLUMN created_on;
ALTER TABLE screenshot DROP COLUMN creator_id;
