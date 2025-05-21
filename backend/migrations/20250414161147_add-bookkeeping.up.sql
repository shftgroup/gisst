-- Add up migration script here
CREATE TABLE IF NOT EXISTS metrics (
  name VARCHAR(64) PRIMARY KEY,
  attrs jsonb,
  int_value bigint,
  float_value double precision,
  last_observed_time timestamptz NOT NULL DEFAULT current_timestamp
);

ALTER TABLE file ADD COLUMN file_compressed_size bigint;
