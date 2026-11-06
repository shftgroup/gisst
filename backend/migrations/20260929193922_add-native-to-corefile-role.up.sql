-- Add up migration script here
ALTER TYPE core_file_role ADD VALUE IF NOT EXISTS 'native' AFTER 'config';
