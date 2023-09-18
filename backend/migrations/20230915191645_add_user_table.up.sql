-- Add up migration script here
CREATE TYPE auth_provider as ENUM ('google','orcid');

CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    creator_id uuid,
    password_hash text not null,
    name text,
    given_name text,
    family_name text,
    preferred_username text,
    email text,
    picture text
);

ALTER TABLE users ADD FOREIGN KEY (creator_id) REFERENCES creator(creator_id);