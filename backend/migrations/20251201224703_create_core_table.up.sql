CREATE TYPE core_file_role AS ENUM ('entrypoint', 'dependency', 'config');

CREATE TABLE IF NOT EXISTS core (
  core_name text NOT NULL,
  core_version text NOT NULL,
  core_metadata jsonb NOT NULL,
  created_on timestamptz DEFAULT current_timestamp NOT NULL,
  PRIMARY KEY (core_name, core_version)
);

-- Automatic migration step: upgrade legacy cores into core table
WITH vars as (SELECT current_timestamp as now)
  INSERT INTO core
    SELECT DISTINCT environment_core_name,
                    environment_core_version,
                    '{"legacy":true}'::jsonb,
                    vars.now
    FROM environment, vars;

CREATE TABLE IF NOT EXISTS core_file (
  core_name text NOT NULL,
  core_version text NOT NULL,
  file_id uuid NOT NULL,
  core_role core_file_role NOT NULL,
  core_role_index integer NOT NULL DEFAULT 0,
  PRIMARY KEY (core_name, core_version, file_id)
);

ALTER TABLE core_file ADD FOREIGN KEY (file_id) REFERENCES file(file_id);
ALTER TABLE core_file ADD FOREIGN KEY (core_name,core_version) REFERENCES core(core_name,core_version);
ALTER TABLE environment ADD FOREIGN KEY (environment_core_name,environment_core_version) REFERENCES core(core_name,core_version);

CREATE INDEX idx_core_key ON core_file(core_name, core_version);

-- Manual migration steps:
-- -- gisst-cli add-core core.json --entrypoint file --dependency file --dependency file --config file ...
-- -- gisst-cli upgrade-environments old-core-name old-core-version core.json
-- It will run sql like the following:
-- -- Manual migration step (needs files not currently in storage): insert seabios, vgabios, etc into core_file for v86.
-- -- You can find the v86 ones with SELECT * FROM instance JOIN environment USING (environment_id) WHERE environment_framework = 'v86';
-- -- Manual migration step: insert scph500X.bin into core_file for pcsx_rearmed:
-- -- Check instanceobject links involving scph:
-- select file.file_id,file.file_filename,instanceobject.* from object join file using(file_id) join instanceobject using(object_id) where file.file_filename like 'scph%';
-- -- Add to core_file:
-- INSERT INTO core_file VALUES
--   (SELECT ('pcsx_rearmed', '1.62.3', file.file_id, 'dependency', instanceobject.object_role_index) FROM file join object using(file_id) join instanceobject using(object_id) WHERE file.file_filename like 'scph%');
-- -- Delete from instanceobject and object:
-- DELETE FROM instanceobject WHERE object_id IN object join file using(file_id) where file.file_filename like 'scph%';
