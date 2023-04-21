-- Add migration script here

INSERT INTO core (core_name, core_version) VALUES ("nes", "0.0.1");
INSERT INTO platform (core_id, platform_framework) VALUES (1, "retroarch");

