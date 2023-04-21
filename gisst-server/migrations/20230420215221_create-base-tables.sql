-- Add migration script here
DROP TABLE content;

CREATE TABLE IF NOT EXISTS content (
    content_id          serial  PRIMARY KEY,
    content_uuid        uuid,
    content_title       text,
    content_version     text,
    content_path        text,
    content_filename    text,
    platform_id integer,
    content_parent_id   integer,
    created_on  timestamp
);

DROP TABLE platform;

CREATE TABLE IF NOT EXISTS platform (
    platform_id         serial PRIMARY KEY,
    core_id             integer,
    platform_framework  text,
    created_on          timestamp
);

DROP TABLE creator;
CREATE TABLE IF NOT EXISTS creator (
    creator_id          serial PRIMARY KEY,
    creator_username    varchar(20),
    creator_full_name   varchar(50),
    created_on  timestamp
);

DROP TABLE save;
CREATE TABLE IF NOT EXISTS save (
    save_id          serial PRIMARY KEY,
    save_uuid        uuid,
    save_short_desc  varchar(100),
    save_description text,
    save_filename    text,
    save_path        text,
    creator_id  integer,
    content_id  integer,
    core_id     integer,
    created_on  timestamp
);

DROP TABLE image;
CREATE TABLE IF NOT EXISTS image (
    image_id          serial PRIMARY KEY,
    image_filename    text,
    image_parent_id   integer,
    image_path        text,
    created_on  timestamp
);

DROP TABLE replay;
CREATE TABLE IF NOT EXISTS replay (
    replay_id           serial PRIMARY KEY,
    content_id          integer,
    save_id             integer,
    replay_forked_from  integer,
    replay_filename     integer,
    replay_path         text,
    core_id             integer
);

DROP TABLE core;
CREATE TABLE IF NOT EXISTS core (
    core_id         serial PRIMARY KEY,
    core_name       text,
    core_version    text,
    core_manifest   json
);

DROP TABLE state;
CREATE TABLE IF NOT EXISTS state (
    state_id            serial PRIMARY KEY,
    state_screenshot    bytea,
    replay_id           integer,
    content_id          integer,
    creator_id          integer,
    state_replay_index  integer,
    is_checkpoint       boolean,
    state_path          text,
    state_filename      text,
    state_name          text,
    state_description   text,
    core_id             integer,
    derived_from        integer
);

-- Add foreign key constraints for all tables
ALTER TABLE content ADD FOREIGN KEY (platform_id) REFERENCES platform(platform_id);
ALTER TABLE content ADD FOREIGN KEY (content_parent_id) REFERENCES content(content_id);

ALTER TABLE platform ADD FOREIGN KEY (core_id) REFERENCES core(core_id);

ALTER TABLE save ADD FOREIGN KEY (creator_id) REFERENCES creator(creator_id);
ALTER TABLE save ADD FOREIGN KEY (content_id) REFERENCES content(content_id);
ALTER TABLE save ADD FOREIGN KEY (core_id) REFERENCES core(core_id);

ALTER TABLE image ADD FOREIGN KEY (image_parent_id) REFERENCES image(image_id);

ALTER TABLE replay ADD FOREIGN KEY (content_id) REFERENCES content(content_id);
ALTER TABLE replay ADD FOREIGN KEY (save_id) REFERENCES save(save_id);
ALTER TABLE replay ADD FOREIGN KEY (replay_forked_from) REFERENCES replay(replay_id);
ALTER TABLE replay ADD FOREIGN KEY (core_id) REFERENCES core(core_id);

ALTER TABLE state ADD FOREIGN KEY (replay_id) REFERENCES replay(replay_id);
ALTER TABLE state ADD FOREIGN KEY (content_id) REFERENCES content(content_id);
ALTER TABLE state ADD FOREIGN KEY (creator_id) REFERENCES creator(creator_id);
ALTER TABLE state ADD FOREIGN KEY (core_id) REFERENCES core(core_id);
ALTER TABLE state ADD FOREIGN KEY (derived_from) REFERENCES state(state_id);
