-- Add up migration script here

UPDATE creator SET created_on = current_timestamp WHERE created_on IS NULL;
UPDATE environment SET created_on = current_timestamp WHERE created_on IS NULL;
UPDATE file SET created_on = current_timestamp WHERE created_on IS NULL;
UPDATE image SET created_on = current_timestamp WHERE created_on IS NULL;
UPDATE instance SET created_on = current_timestamp WHERE created_on IS NULL;
UPDATE object SET created_on = current_timestamp WHERE created_on IS NULL;
UPDATE replay SET created_on = current_timestamp WHERE created_on IS NULL;
UPDATE save SET created_on = current_timestamp WHERE created_on IS NULL;
UPDATE state SET created_on = current_timestamp WHERE created_on IS NULL;
UPDATE work SET created_on = current_timestamp WHERE created_on IS NULL;

ALTER TABLE IF EXISTS creator ALTER COLUMN created_on SET NOT NULL;
ALTER TABLE IF EXISTS users ALTER COLUMN creator_id SET NOT NULL;
ALTER TABLE IF EXISTS environment ALTER COLUMN created_on SET NOT NULL;
ALTER TABLE IF EXISTS file ALTER COLUMN created_on SET NOT NULL;
ALTER TABLE IF EXISTS image ALTER COLUMN created_on SET NOT NULL;
ALTER TABLE IF EXISTS instance ALTER COLUMN created_on SET NOT NULL;
ALTER TABLE IF EXISTS object ALTER COLUMN created_on SET NOT NULL;
ALTER TABLE IF EXISTS replay ALTER COLUMN created_on SET NOT NULL;
ALTER TABLE IF EXISTS save ALTER COLUMN created_on SET NOT NULL;
ALTER TABLE IF EXISTS state ALTER COLUMN created_on SET NOT NULL;
ALTER TABLE IF EXISTS state ALTER COLUMN creator_id SET NOT NULL;
ALTER TABLE IF EXISTS state ALTER COLUMN screenshot_id SET NOT NULL;
ALTER TABLE IF EXISTS work ALTER COLUMN created_on SET NOT NULL;
