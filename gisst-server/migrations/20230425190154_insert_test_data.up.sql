-- Add up migration script here
DO $$
    DECLARE uuid_0 uuid;
    DECLARE uuid_1 uuid;
    DECLARE uuid_2 uuid;
BEGIN
        uuid_0 = '00000000-0000-0000-0000-000000000000'::UUID;
        uuid_1 = '00000000-0000-0000-0000-000000000001'::UUID;
        uuid_2 = '00000000-0000-0000-0000-000000000002'::UUID;
-- Insert balloon fight NES information into the database

        INSERT INTO core(core_id, core_name, core_version)
        VALUES(uuid_0, 'nestopia_libretro_emscripten', '1.52.0');

        INSERT INTO platform(platform_id, core_id, platform_name, platform_framework)
        VALUES(uuid_0, uuid_0, 'Nintendo Entertainment System', 'retroarch');

        INSERT INTO content(content_id, content_hash, content_title, content_version, content_source_filename, content_dest_filename, platform_id)
        VALUES (uuid_0, '6e125395ca4f18addb8ce6c9152dea85', 'Balloon Fight (USA)', 'NTSC', 'bfight.nes', '6e125395ca4f18addb8ce6c9152dea85-bfight.nes', uuid_0);

        INSERT INTO content(content_id, content_hash, content_title, content_source_filename, content_dest_filename, platform_id, content_parent_id)
        VALUES (uuid_1, 'a028e5747a6d0a658060644f56663d51', 'retroarch.cfg', 'retroarch.cfg', 'a028e5747a6d0a658060644f56663d51-retroarch.cfg', uuid_0, uuid_0);

        INSERT INTO replay(replay_id, content_id, replay_filename, replay_hash, core_id)
        VALUES (uuid_0, uuid_0, 'bfight.replay', 'b9504a4016f39757fffeccdd79283694', uuid_0);

        INSERT INTO state(state_id, content_id, state_filename, state_hash, state_name, state_description, core_id)
        VALUES (uuid_0, uuid_0, 'bfight.entry_state', '6c7a3d2e61eefef3fbd1da6d194be1c2', 'Balloon Fight Entry State', 'A test entry state for the game Balloon Fight', uuid_0);


-- Insert for v86 freedos games

        INSERT INTO image

    END $$;
