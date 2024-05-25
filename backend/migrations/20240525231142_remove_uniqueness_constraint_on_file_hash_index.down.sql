-- Add down migration script here
DROP INDEX idx_file_hash;
CREATE UNIQUE INDEX idx_file_hash ON file(file_hash);
