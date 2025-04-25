ALTER TABLE save
  DROP COLUMN IF EXISTS state_derived_from;
ALTER TABLE save
  DROP COLUMN IF EXISTS save_derived_from;
ALTER TABLE save
  DROP COLUMN IF EXISTS replay_derived_from;


DROP INDEX IF EXISTS save_derived_state_idx;
DROP INDEX IF EXISTS save_save_idx;
DROP INDEX IF EXISTS save_replay_idx;
DROP INDEX IF EXISTS save_instance_idx;
DROP INDEX IF EXISTS save_creator_idx;

DROP TABLE IF EXISTS instance_save;

DROP INDEX IF EXISTS instance_save_instance;
DROP INDEX IF EXISTS instance_save_save;
