-- Add down migration script here
DELETE FROM content WHERE content_id = 1;
DELETE FROM platform WHERE platform_id = 1;
DELETE FROM core WHERE core_id = 1;
