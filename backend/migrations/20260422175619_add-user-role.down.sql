-- Add down migration script here

ALTER TABLE users DROP COLUMN user_role;

DROP FUNCTION public.f_user_role_superuser;
DROP FUNCTION public.f_user_role_admin;
DROP FUNCTION public.f_user_role_regular;
DROP FUNCTION public.f_user_role_visitor;
