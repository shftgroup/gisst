-- Add migration script here

INSERT INTO core (core_name, core_version) VALUES ("nes", "0.0.1");
INSERT INTO platform (core_id, platform_framework) VALUES (1, "retroarch");
INSERT INTO content (content_title, content_version, content_path, content_filename, platform_id) VALUES ("super mario", "0.1", "hashvalue/human_readable", "test",1);

