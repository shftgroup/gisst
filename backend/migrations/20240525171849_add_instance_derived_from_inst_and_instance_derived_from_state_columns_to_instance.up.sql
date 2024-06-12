-- Add up migration script here
ALTER TABLE instance
ADD COLUMN derived_from_instance uuid,
ADD COLUMN derived_from_state uuid;
