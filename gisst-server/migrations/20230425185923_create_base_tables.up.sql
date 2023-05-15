-- Add up migration script here

CREATE TABLE IF NOT EXISTS content (
                                       content_id        uuid PRIMARY KEY DEFAULT gen_random_uuid(),
                                       content_hash        bytea,
                                       content_title       text,
                                       content_version     text,
                                       content_path        text,
                                       content_filename    text,
                                       platform_id          uuid,
                                       content_parent_id   uuid,
                                       created_on  timestamptz DEFAULT current_timestamp
);


CREATE TABLE IF NOT EXISTS platform (
                                        platform_id       uuid PRIMARY KEY DEFAULT gen_random_uuid(),
                                        core_id             uuid,
                                        platform_framework  text,
                                        created_on          timestamptz DEFAULT current_timestamp
);

CREATE TABLE IF NOT EXISTS creator (
                                       creator_id        uuid PRIMARY KEY DEFAULT gen_random_uuid(),
                                       creator_username    varchar(20),
                                       creator_full_name   varchar(50),
                                       created_on  timestamptz DEFAULT current_timestamp
);

CREATE TABLE IF NOT EXISTS save (
                                    save_id        uuid PRIMARY KEY DEFAULT gen_random_uuid(),
                                    save_short_desc  varchar(100),
                                    save_description text,
                                    save_filename    text,
                                    save_path        text,
                                    creator_id     uuid,
                                    content_id     uuid,
                                    core_id          uuid,
                                    created_on  timestamptz DEFAULT current_timestamp
);

CREATE TABLE IF NOT EXISTS image (
                                     image_id        uuid PRIMARY KEY DEFAULT gen_random_uuid(),
                                     image_filename    text,
                                     image_parent_id   uuid,
                                     image_path        text,
                                     image_hash        bytea,
                                     created_on  timestamptz DEFAULT current_timestamp
);

CREATE TABLE IF NOT EXISTS replay (
                                      replay_id           uuid PRIMARY KEY DEFAULT gen_random_uuid(),
                                      content_id          uuid,
                                      save_id             uuid,
                                      replay_forked_from  uuid,
                                      replay_filename     integer,
                                      replay_path         text,
                                      core_id            uuid
);

CREATE TABLE IF NOT EXISTS core (
                                    core_id       uuid PRIMARY KEY DEFAULT gen_random_uuid(),
                                    core_name       text,
                                    core_version    text,
                                    core_manifest   json
);

CREATE TABLE IF NOT EXISTS state (
                                     state_id            uuid PRIMARY KEY DEFAULT gen_random_uuid(),
                                     screenshot_id       uuid,
                                     replay_id           uuid,
                                     content_id          uuid,
                                     creator_id          uuid,
                                     state_replay_index  integer,
                                     is_checkpoint       boolean,
                                     state_path          text,
                                     state_filename      text,
                                     state_name          text,
                                     state_description   text,
                                     core_id             uuid,
                                     state_derived_from        uuid
);

CREATE TABLE IF NOT EXISTS screenshot (
    screenshot_id   uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    screenshot_data bytea
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
ALTER TABLE state ADD FOREIGN KEY (state_derived_from) REFERENCES state(state_id);
ALTER TABLE state ADD FOREIGN KEY (screenshot_id) REFERENCES screenshot(screenshot_id);

-- Add indexes for specific fields
CREATE UNIQUE INDEX idx_content_hash ON content(content_hash);