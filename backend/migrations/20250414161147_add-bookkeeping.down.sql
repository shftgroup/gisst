-- Add down migration script here
DROP TABLE metrics;
ALTER TABLE FILE DROP COLUMN file_compressed_size;
