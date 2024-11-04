CREATE EXTENSION IF NOT EXISTS unaccent;
CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE OR REPLACE FUNCTION immutable_unaccent(regdictionary, text)
  RETURNS text
  LANGUAGE c IMMUTABLE PARALLEL SAFE STRICT AS
'$libdir/unaccent', 'unaccent_dict';

CREATE OR REPLACE FUNCTION f_unaccent(text)
  RETURNS text
  LANGUAGE sql IMMUTABLE PARALLEL SAFE STRICT
RETURN immutable_unaccent(regdictionary 'public.unaccent', $1);

CREATE MATERIALIZED VIEW instanceWork (row_num, work_id, work_name, work_version, work_platform, instance_id)
  AS SELECT row_number() OVER (ORDER BY f_unaccent(lower(work.work_name)) ASC, f_unaccent(lower(work.work_version)) ASC, f_unaccent(lower(work.work_platform)) ASC, instance.instance_id ASC),
            work.work_id, work.work_name, work.work_version, work.work_platform, instance.instance_id
     FROM work JOIN instance USING(work_id)
  WITH DATA;


CREATE OR REPLACE FUNCTION updateInstanceWork()
  RETURNS trigger
  LANGUAGE plpgsql
  AS $$
  BEGIN
    REFRESH MATERIALIZED VIEW instanceWork WITH DATA;
    RETURN NULL;
  END $$;

CREATE TRIGGER instance_insert_updateInstanceWork AFTER INSERT ON instance
  FOR EACH STATEMENT
  EXECUTE PROCEDURE updateInstanceWork();

CREATE INDEX instanceWorkSearch ON instanceWork
  USING gin (f_unaccent(work_name) gin_trgm_ops);
CREATE UNIQUE INDEX instanceWorkWhich ON instanceWork (row_num);
