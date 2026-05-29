-- Add up migration script here
ALTER TABLE core ADD COLUMN core_platform text NOT NULL DEFAULT '';

UPDATE core
  SET core_platform = work.work_platform
  FROM environment, instance, work
  WHERE core.core_name = environment.environment_core_name AND
        environment.environment_id = instance.environment_id AND
	instance.work_id = work.work_id;

UPDATE core
  SET core_platform = 'Nintendo Game Boy Advance'
  WHERE core.core_name = 'vba_next';

UPDATE core
  SET core_platform = 'Nintendo Game Boy'
  WHERE core.core_name = 'gambatte' OR core.core_name = 'sameboy';

UPDATE core
  SET core_platform = 'Nintendo Entertainment System'
  WHERE core.core_name = 'fceumm';

UPDATE core
  SET core_platform = 'Super Nintendo Entertainment System'
  WHERE core.core_name = 'snes9x';

UPDATE core
  SET core_platform = 'Nintendo 64'
  WHERE core.core_name = 'mupen64plus_next';

UPDATE core
  SET core_platform = 'Sony Playstation'
  WHERE core.core_name = 'pcsx_rearmed';

UPDATE core
  SET core_platform = 'Commodore 64'
  WHERE core.core_name = 'vice_x64';

UPDATE core
  SET core_platform = 'x86 PC'
  WHERE core.core_name = 'v86';

