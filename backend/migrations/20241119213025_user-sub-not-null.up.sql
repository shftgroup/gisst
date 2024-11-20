-- Add up migration script here
ALTER TABLE users ALTER COLUMN sub SET NOT NULL;
ALTER TABLE users ADD COLUMN iss text NOT NULL DEFAULT 'https://accounts.google.com';
ALTER TABLE users ADD CONSTRAINT no_duplicate_sub UNIQUE(iss,sub);
