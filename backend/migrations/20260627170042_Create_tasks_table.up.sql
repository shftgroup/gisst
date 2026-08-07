CREATE TYPE task_state AS ENUM ('idle', 'active', 'error', 'cancel', 'done');

CREATE TABLE IF NOT EXISTS task (
  task_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  task_created_on timestamptz NOT NULL DEFAULT current_timestamp,
  task_retry_count integer NOT NULL DEFAULT 0,
  task_type text NOT NULL,
  task_claimant text,
  task_claimed_on timestamptz,
  task_updated_on timestamptz NOT NULL DEFAULT current_timestamp,
  task_state task_state NOT NULL DEFAULT 'idle',
  task_status jsonb NOT NULL DEFAULT jsonb('{}'),
  task_last_status jsonb,
  task_input jsonb NOT NULL DEFAULT jsonb('{}'),
  task_output jsonb NOT NULL DEFAULT jsonb('{}')
);
