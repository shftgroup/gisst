-- Add down migration script here

-- Cannot remove data with constraints enabled, so we remove them and put them back
-- to not conflict with create_base_tables.down.sql
ALTER TABLE replay DROP CONSTRAINT replay_creator_id_fkey;
ALTER TABLE save DROP CONSTRAINT save_creator_id_fkey;
ALTER TABLE state DROP CONSTRAINT state_creator_id_fkey;
ALTER TABLE users DROP CONSTRAINT users_creator_id_fkey;

TRUNCATE creator;
ALTER TABLE IF EXISTS creator ALTER COLUMN creator_username SET DATA TYPE varchar(20);
ALTER TABLE IF EXISTS creator ALTER COLUMN creator_full_name SET DATA TYPE varchar(50);


ALTER TABLE save ADD FOREIGN KEY (creator_id) REFERENCES creator(creator_id);
ALTER TABLE replay ADD FOREIGN KEY (creator_id) REFERENCES creator(creator_id);
ALTER TABLE state ADD FOREIGN KEY (creator_id) REFERENCES creator(creator_id);
ALTER TABLE users ADD FOREIGN KEY (creator_id) REFERENCES creator(creator_id);
