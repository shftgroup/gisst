-- Add down migration script here

ALTER TABLE platform DROP CONSTRAINT platform_core_id_fkey;
ALTER TABLE content DROP CONSTRAINT content_platform_id_fkey;
ALTER TABLE image DROP CONSTRAINT image_image_parent_id_fkey;
ALTER TABLE replay DROP CONSTRAINT replay_content_id_fkey;
ALTER TABLE replay DROP CONSTRAINT replay_core_id_fkey;
ALTER TABLE replay DROP CONSTRAINT replay_replay_forked_from_fkey;
ALTER TABLE replay DROP CONSTRAINT replay_save_id_fkey;
ALTER TABLE save DROP CONSTRAINT save_content_id_fkey;
ALTER TABLE save DROP CONSTRAINT save_core_id_fkey;
ALTER TABLE save DROP CONSTRAINT save_creator_id_fkey;
ALTER TABLE state DROP CONSTRAINT state_content_id_fkey;
ALTER TABLE state DROP CONSTRAINT state_core_id_fkey;
ALTER TABLE state DROP CONSTRAINT state_creator_id_fkey;
ALTER TABLE state DROP CONSTRAINT state_state_derived_from_fkey;
ALTER TABLE state DROP CONSTRAINT state_replay_id_fkey;
ALTER TABLE state DROP CONSTRAINT state_screenshot_id_fkey;

DROP TABLE platform;
DROP TABLE content;
DROP TABLE save;
DROP TABLE image;
DROP TABLE replay;
DROP TABLE state;
DROP TABLE creator;
DROP TABLE core;
DROP TABLE screenshot;
