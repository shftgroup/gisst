-- Add up migration script here
INSERT INTO core (core_name, core_version) VALUES ('nes', '0.1');
INSERT INTO platform (core_id, platform_framework) VALUES (1, 'retroarch');
INSERT INTO content (content_uuid, content_title, content_version, content_path, content_filename, platform_id)
VALUES (gen_random_uuid(), 'test_object','0.1', 'hash', 'test_file', 1);