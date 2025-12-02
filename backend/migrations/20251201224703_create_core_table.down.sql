ALTER TABLE environment DROP CONSTRAINT environment_core_name_fkey;
ALTER TABLE environment DROP CONSTRAINT environment_core_version_fkey;
DROP INDEX IF EXISTS idx_core_key;
DROP TABLE core_file;
DROP TABLE core;
DROP TYPE core_file_role;
