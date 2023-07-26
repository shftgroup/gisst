-- Add down migration script here

ALTER TABLE environment DROP CONSTRAINT environment_environment_derived_from_fkey;

ALTER TABLE environmentImage DROP CONSTRAINT environmentImage_environment_id_fkey;
ALTER TABLE environmentImage DROP CONSTRAINT environmentImage_image_id_fkey;

ALTER TABLE image DROP CONSTRAINT image_file_id_fkey;

ALTER TABLE instance DROP CONSTRAINT instance_environment_id_fkey;
ALTER TABLE instance DROP CONSTRAINT instance_work_id_fkey;

ALTER TABLE instanceObject DROP CONSTRAINT instanceObject_instance_id_fkey;
ALTER TABLE instanceObject DROP CONSTRAINT instanceObject_object_id_fkey;

ALTER TABLE object DROP CONSTRAINT object_file_id_fkey;

ALTER TABLE replay DROP CONSTRAINT replay_instance_id_fkey;
ALTER TABLE replay DROP CONSTRAINT replay_replay_forked_from_fkey;
ALTER TABLE replay DROP CONSTRAINT replay_creator_id_fkey;
ALTER TABLE replay DROP CONSTRAINT replay_file_id_fkey;

ALTER TABLE save DROP CONSTRAINT save_instance_id_fkey;
ALTER TABLE save DROP CONSTRAINT save_creator_id_fkey;
ALTER TABLE save DROP CONSTRAINT save_file_id_fkey;

ALTER TABLE state DROP CONSTRAINT state_instance_id_fkey;
ALTER TABLE state DROP CONSTRAINT state_creator_id_fkey;
ALTER TABLE state DROP CONSTRAINT state_state_derived_from_fkey;
ALTER TABLE state DROP CONSTRAINT state_replay_id_fkey;
ALTER TABLE state DROP CONSTRAINT state_screenshot_id_fkey;
ALTER TABLE state DROP CONSTRAINT state_file_id_fkey;

DROP TABLE creator;
DROP TABLE environment;
DROP TABLE environmentImage;
DROP TABLE instance;
DROP TABLE instanceObject;
DROP TABLE object;
DROP TABLE save;
DROP TABLE state;
DROP TABLE screenshot;
DROP TABLE image;
DROP TABLE replay;
DROP TABLE work;
DROP TABLE file;
DROP TYPE IF EXISTS object_role;
DROP TYPE IF EXISTS environment_framework;
