ALTER TABLE save
  ADD COLUMN IF NOT EXISTS
  associated_state uuid NOT NULL;
ALTER TABLE save
  ADD COLUMN IF NOT EXISTS
  state_derived_from uuid;
ALTER TABLE save
  ADD COLUMN IF NOT EXISTS
  save_derived_from uuid;
ALTER TABLE save
  ADD COLUMN IF NOT EXISTS
  replay_derived_from uuid;

ALTER TABLE state
  ADD COLUMN IF NOT EXISTS
  save_derived_from uuid;

ALTER TABLE save ADD FOREIGN KEY (associated_state) REFERENCES state(state_id);
ALTER TABLE save ADD FOREIGN KEY (state_derived_from) REFERENCES state(state_id);
ALTER TABLE save ADD FOREIGN KEY (save_derived_from) REFERENCES save(save_id);
ALTER TABLE save ADD FOREIGN KEY (replay_derived_from) REFERENCES replay(replay_id);
ALTER TABLE state ADD FOREIGN KEY (state_derived_from) REFERENCES state(state_id);

CREATE INDEX IF NOT EXISTS save_state_idx on save (associated_state);
CREATE INDEX IF NOT EXISTS save_derived_state_idx on save (state_derived_from);
CREATE INDEX IF NOT EXISTS save_save_idx on save (save_derived_from);
CREATE INDEX IF NOT EXISTS save_replay_idx on save (replay_derived_from);
CREATE INDEX IF NOT EXISTS save_instance_idx on save (instance_id);
CREATE INDEX IF NOT EXISTS save_creator_idx on save (creator_id);

CREATE TABLE IF NOT EXISTS instance_save (
       instance_id uuid NOT NULL,
       save_id uuid NOT NULL,
       PRIMARY KEY (instance_id, save_id)
);
ALTER TABLE instance_save ADD FOREIGN KEY (instance_id) REFERENCES instance(instance_id);
ALTER TABLE instance_save ADD FOREIGN KEY (save_id) REFERENCES save(save_id);

CREATE INDEX IF NOT EXISTS instance_save_instance on instance_save (instance_id);
CREATE INDEX IF NOT EXISTS instance_save_save on instance_save (save_id);
