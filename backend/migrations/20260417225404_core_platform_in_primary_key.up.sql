-- Add up migration script here
ALTER TABLE core_file DROP CONSTRAINT core_file_core_name_core_version_fkey;
ALTER TABLE environment DROP CONSTRAINT environment_environment_core_name_environment_core_version_fkey;
ALTER TABLE environment ADD COLUMN environment_platform text;
UPDATE environment
  SET environment_platform=core.core_platform
  FROM core
  WHERE environment.environment_core_name = core.core_name;
ALTER TABLE environment ALTER COLUMN environment_platform SET NOT NULL;
ALTER TABLE core DROP CONSTRAINT core_pkey;
ALTER TABLE core ADD CONSTRAINT core_pkey primary key (core_name, core_version, core_platform);
ALTER TABLE environment ADD FOREIGN KEY (environment_core_name,environment_core_version,environment_platform) REFERENCES core(core_name,core_version,core_platform);
