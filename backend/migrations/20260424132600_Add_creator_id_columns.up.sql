-- Add up migration script here

ALTER TABLE work ADD COLUMN creator_id uuid;
ALTER TABLE environment ADD COLUMN creator_id uuid;
ALTER TABLE instance ADD COLUMN creator_id uuid;
ALTER TABLE object ADD COLUMN creator_id uuid;
ALTER TABLE file ADD COLUMN creator_id uuid;
ALTER TABLE screenshot ADD COLUMN created_on timestamptz NOT NULL default current_timestamp;
ALTER TABLE screenshot ADD COLUMN creator_id uuid;
