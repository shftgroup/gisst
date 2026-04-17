-- Add down migration script here
DELETE FROM core WHERE (core_name, core_version, core_platform) NOT IN (
   SELECT DISTINCT ON (core_name, core_version)
          core_name, core_version, core_platform
   FROM core
   ORDER BY core_name, core_version, core_platform);

ALTER TABLE environment DROP CONSTRAINT environment_environment_core_name_environment_core_version_environment_platform_fkey;
ALTER TABLE environment DROP COLUMN environment_platform;

ALTER TABLE core DROP CONSTRAINT core_pkey;
ALTER TABLE core ADD CONSTRAINT core_pkey primary key (core_name, core_version);
ALTER TABLE environment ADD FOREIGN KEY (environment_core_name,environment_core_version) REFERENCES core(core_name,core_version);


