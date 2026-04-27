
DROP TRIGGER IF EXISTS  instance_insert_updateInstanceWork ON instance;
DROP FUNCTION IF EXISTS updateInstanceWork;
DROP INDEX IF EXISTS instanceWorkSearch;
DROP INDEX IF EXISTS instanceWorkWhich;
DROP MATERIALIZED VIEW IF EXISTS instanceWork;
DROP FUNCTION IF EXISTS f_unaccent;
DROP FUNCTION IF EXISTS immutable_unaccent;
