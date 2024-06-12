-- Add up migration script here
DROP INDEX idx_file_hash;
CREATE INDEX idx_file_hash ON file(file_hash);
