-- Add up migration script here
DO $$
DECLARE lastplatformid platform.platform_id%TYPE;
BEGIN
INSERT INTO core (core_name, core_version) VALUES ('nes', '0.1');
INSERT INTO platform (platform_framework) VALUES ('retroarch') RETURNING platform_id INTO lastplatformid;
INSERT INTO content (content_title, content_version, content_path, content_filename, platform_id)
VALUES ('test_object','0.1', 'hash', 'test_file', lastplatformid);
END $$;