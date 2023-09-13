# Remove local file storage
rm -r storage
# Remove test_data load
sqlx migrate revert
# Remove tables and data
sqlx migrate revert
# Recreate tables and data load
sqlx migrate run

export GISST_CONFIG_PATH=./config
uuid_0=00000000000000000000000000000000
uuid_1=00000000000000000000000000000001
uuid_2=00000000000000000000000000000002
uuid_3=00000000000000000000000000000003
content_path=./test_content
cargo build --bin gisst-cli
./target/debug/gisst-cli creator create --json-string '{"creator_id":"00000000000000000000000000000000","creator_username":"Joe","creator_full_name":"Joe"}'
./target/debug/gisst-cli work create --json-file ${content_path}/test_bfight_work.json
./target/debug/gisst-cli environment create --json-file ${content_path}/test_bfight_environment.json
./target/debug/gisst-cli instance create --json-file ${content_path}/test_bfight_instance.json
./target/debug/gisst-cli object create -i --force-uuid $uuid_0 --link $uuid_0 --role content ${content_path}/bfight.nes
./target/debug/gisst-cli object create -i --force-uuid $uuid_1 --link $uuid_0 --role config ${content_path}/retroarch.cfg
./target/debug/gisst-cli state create --force-uuid $uuid_0 --link $uuid_0 --file ${content_path}/bfight.entry_state --name "Balloon Fight Test State"
./target/debug/gisst-cli replay create --force-uuid $uuid_0 --link $uuid_0 --file ${content_path}/bfight.replay1

./target/debug/gisst-cli work create --json-file ${content_path}/test_snake_work.json
./target/debug/gisst-cli environment create --json-file ${content_path}/test_snake_environment.json --environment-config-string '{"bios":{"url":"seabios.bin"},"vga_bios":{"url":"vgabios.bin"},"fda":{"url":"$CONTENT","async":true}}'
./target/debug/gisst-cli instance create --json-file ${content_path}/test_snake_instance.json
./target/debug/gisst-cli object create -i --force-uuid $uuid_2 --link $uuid_1 --role content ${content_path}/freedos722.img
./target/debug/gisst-cli state create --force-uuid $uuid_1 --link $uuid_1 --file ${content_path}/state0.v86state --name "Snake Test State"
./target/debug/gisst-cli replay create --force-uuid $uuid_1 --link $uuid_1 --file ${content_path}/replay0.v86replay
