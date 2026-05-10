-- Add up migration script here

CREATE FUNCTION public.f_user_role_visitor()
  RETURNS int
  LANGUAGE sql IMMUTABLE PARALLEL SAFE AS
'SELECT 100';

CREATE FUNCTION public.f_user_role_regular()
  RETURNS int
  LANGUAGE sql IMMUTABLE PARALLEL SAFE AS
'SELECT 50';

CREATE FUNCTION public.f_user_role_admin()
  RETURNS int
  LANGUAGE sql IMMUTABLE PARALLEL SAFE AS
'SELECT 10';

CREATE FUNCTION public.f_user_role_superuser()
  RETURNS int
  LANGUAGE sql IMMUTABLE PARALLEL SAFE AS
'SELECT 0';

ALTER TABLE users ADD COLUMN user_role integer NOT NULL DEFAULT(public.f_user_role_regular());
