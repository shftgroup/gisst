-- Add up migration script here
CREATE TYPE object_role AS ENUM ('content', 'dependency', 'config');
CREATE TYPE environment_framework AS ENUM ('retroarch', 'v86');

CREATE TABLE IF NOT EXISTS creator (
    creator_id        uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    creator_username    varchar(20) NOT NULL,
    creator_full_name   varchar(50) NOT NULL,
    created_on  timestamptz DEFAULT current_timestamp
);

CREATE TABLE IF NOT EXISTS environment (
    environment_id                  uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    environment_name                text NOT NULL,
    environment_framework           environment_framework NOT NULL,
    environment_core_name           text NOT NULL,
    environment_core_version        text NOT NULL,
    environment_derived_from        uuid,
    environment_config              jsonb,
    created_on                      timestamptz DEFAULT current_timestamp
);

CREATE TABLE IF NOT EXISTS environmentImage (
    environment_id              uuid,
    image_id                    uuid,
    environment_image_config    jsonb,
    PRIMARY KEY (environment_id, image_id)
);

CREATE TABLE IF NOT EXISTS instance (
    instance_id             uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    environment_id          uuid NOT NULL,
    work_id                 uuid NOT NULL,
    instance_config         jsonb,
    created_on              timestamptz DEFAULT current_timestamp
);

CREATE TABLE IF NOT EXISTS instanceObject (
    instance_id             uuid NOT NULL,
    object_id               uuid NOT NULL,
    object_role             object_role NOT NULL,
    instance_object_config  jsonb,
    PRIMARY KEY (instance_id, object_id)
);

CREATE TABLE IF NOT EXISTS object (
    object_id               uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    object_hash             text NOT NULL,
    object_filename         text NOT NULL,
    object_source_path      text NOT NULL,
    object_dest_path        text NOT NULL,
    object_description      text,
    created_on  timestamptz DEFAULT current_timestamp
);

CREATE TABLE IF NOT EXISTS save (
    save_id             uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    instance_id         uuid NOT NULL,
    save_short_desc     varchar(100) NOT NULL,
    save_description    text NOT NULL,
    save_filename       text NOT NULL,
    save_hash           text NOT NULL,
    save_path           text NOT NULL,
    creator_id          uuid NOT NULL,
    created_on          timestamptz DEFAULT current_timestamp
);

CREATE TABLE IF NOT EXISTS image (
    image_id            uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    image_filename      text NOT NULL,
    image_source_path   text NOT NULL,
    image_dest_path     text NOT NULL,
    image_hash          text NOT NULL,
    image_description   text,
    image_config        jsonb,
    created_on  timestamptz DEFAULT current_timestamp
);

CREATE TABLE IF NOT EXISTS replay (
    replay_id           uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    instance_id         uuid NOT NULL,
    creator_id          uuid NOT NULL,
    replay_forked_from  uuid,
    replay_filename     text NOT NULL,
    replay_hash         text NOT NULL,
    replay_path         text NOT NULL,
    created_on          timestamptz DEFAULT current_timestamp
);

CREATE TABLE IF NOT EXISTS screenshot (
    screenshot_id   uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    screenshot_data bytea
);

CREATE TABLE IF NOT EXISTS state (
    state_id            uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    instance_id         uuid NOT NULL,
    is_checkpoint       boolean NOT NULL DEFAULT false,
    state_filename      text NOT NULL,
    state_path          text NOT NULL,
    state_hash          text NOT NULL,
    state_name          text NOT NULL,
    state_description   text NOT NULL,
    screenshot_id       uuid,
    replay_id           uuid,
    creator_id          uuid,
    state_replay_index  integer,
    state_derived_from  uuid,
    created_on          timestamptz DEFAULT current_timestamp
);

CREATE TABLE IF NOT EXISTS work (
    work_id         uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    work_name       text NOT NULL,
    work_version    text NOT NULL,
    work_platform   text NOT NULL,
    created_on      timestamptz DEFAULT current_timestamp
);


-- Add foreign key constraints for all tables
ALTER TABLE environment ADD FOREIGN KEY (environment_derived_from) REFERENCES environment(environment_id);

ALTER TABLE environmentImage ADD FOREIGN KEY (environment_id) REFERENCES environment(environment_id) ON DELETE CASCADE;
ALTER TABLE environmentImage ADD FOREIGN KEY (image_id) REFERENCES image(image_id) ON DELETE CASCADE;

ALTER TABLE instance ADD FOREIGN KEY (environment_id) REFERENCES environment(environment_id) ON DELETE CASCADE;
ALTER TABLE instance ADD FOREIGN KEY (work_id) REFERENCES work(work_id) ON DELETE CASCADE;

ALTER TABLE instanceObject ADD FOREIGN KEY (instance_id) REFERENCES instance(instance_id) ON DELETE CASCADE;
ALTER TABLE instanceObject ADD FOREIGN KEY (object_id) REFERENCES object(object_id) ON DELETE CASCADE;

ALTER TABLE save ADD FOREIGN KEY (creator_id) REFERENCES creator(creator_id);
ALTER TABLE save ADD FOREIGN KEY (instance_id) REFERENCES instance(instance_id) ON DELETE CASCADE;


ALTER TABLE replay ADD FOREIGN KEY (instance_id) REFERENCES instance(instance_id) ON DELETE CASCADE;
ALTER TABLE replay ADD FOREIGN KEY (replay_forked_from) REFERENCES replay(replay_id) ON DELETE CASCADE;
ALTER TABLE replay ADD FOREIGN KEY (creator_id) REFERENCES creator(creator_id);

ALTER TABLE state ADD FOREIGN KEY (replay_id) REFERENCES replay(replay_id);
ALTER TABLE state ADD FOREIGN KEY (instance_id) REFERENCES instance(instance_id) ON DELETE CASCADE;
ALTER TABLE state ADD FOREIGN KEY (creator_id) REFERENCES creator(creator_id);
ALTER TABLE state ADD FOREIGN KEY (state_derived_from) REFERENCES state(state_id) ON DELETE CASCADE;
ALTER TABLE state ADD FOREIGN KEY (screenshot_id) REFERENCES screenshot(screenshot_id);

-- Add indexes for specific fields
CREATE UNIQUE INDEX idx_object_hash ON object(object_hash);
CREATE UNIQUE INDEX idx_image_hash ON image(image_hash);
