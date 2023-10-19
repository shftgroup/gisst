#!/bin/sh

export GISST_CONFIG_PATH=../config
uuid_0=00000000000000000000000000000000
uuid_1=00000000000000000000000000000001
uuid_2=00000000000000000000000000000002
uuid_3=00000000000000000000000000000003
uuid_4=00000000000000000000000000000004
uuid_5=00000000000000000000000000000005
uuid_6=00000000000000000000000000000006
uuid_7=00000000000000000000000000000007
uuid_8=00000000000000000000000000000008
uuid_9=00000000000000000000000000000009
uuid_10=00000000000000000000000000000010
uuid_11=00000000000000000000000000000011
uuid_12=00000000000000000000000000000012
uuid_13=00000000000000000000000000000013
uuid_14=00000000000000000000000000000014
uuid_15=00000000000000000000000000000015
uuid_16=00000000000000000000000000000016
uuid_17=00000000000000000000000000000017
uuid_18=00000000000000000000000000000018
uuid_19=00000000000000000000000000000019
uuid_20=00000000000000000000000000000020
uuid_21=00000000000000000000000000000021
uuid_22=00000000000000000000000000000022
uuid_23=00000000000000000000000000000023

# Build most recent GISST Command Line Interface
cargo build --bin gisst-cli

# Create creator and initial works
../target/debug/gisst-cli creator create --json-string '{"creator_id":"00000000000000000000000000000000","creator_username":"GISST","creator_full_name":"GISST Test Script"}'
../target/debug/gisst-cli work create --json-file ./records/nes/240p_test_suite_work.json
../target/debug/gisst-cli work create --json-file ./records/nes/alter_ego_work.json
../target/debug/gisst-cli work create --json-file ./records/nes/bobl_work.json
../target/debug/gisst-cli work create --json-file ./records/nes/chrono_knight_work.json
../target/debug/gisst-cli work create --json-file ./records/nes/curse_of_the_shadow_work.json
../target/debug/gisst-cli work create --json-file ./records/nes/forgotten_in_time_work.json
../target/debug/gisst-cli work create --json-file ./records/snes/240p_test_suite_SNES_work.json
../target/debug/gisst-cli work create --json-file ./records/snes/keeping_SNES_alive_work.json
../target/debug/gisst-cli work create --json-file ./records/snes/n_warp_daisakusen_europe_work.json
../target/debug/gisst-cli work create --json-file ./records/snes/super_boss_gaiden_japan_work.json
../target/debug/gisst-cli work create --json-file ./records/v86/snake_work.json

# Create environments
../target/debug/gisst-cli environment create --json-file ./records/nes/nes_fceumm_1_52_environment.json
../target/debug/gisst-cli environment create --json-file ./records/snes/snes_snes9x_1_62_3_environment.json
../target/debug/gisst-cli environment create --json-file ./records/v86/freedos_environment.json --environment-config-string '{"bios":{"url":"seabios.bin"},"vga_bios":{"url":"vgabios.bin"},"fda":{"url":"$CONTENT","async":true}}'

# Create instances
../target/debug/gisst-cli instance create --json-file ./records/nes/240p_test_suite_instance.json
../target/debug/gisst-cli instance create --json-file ./records/nes/alter_ego_instance.json
../target/debug/gisst-cli instance create --json-file ./records/nes/bobl_instance.json
../target/debug/gisst-cli instance create --json-file ./records/nes/chrono_knight_instance.json
../target/debug/gisst-cli instance create --json-file ./records/nes/curse_of_the_shadow_instance.json
../target/debug/gisst-cli instance create --json-file ./records/nes/forgotten_in_time_instance.json
../target/debug/gisst-cli instance create --json-file ./records/nes/forgotten_in_time_instance.json
../target/debug/gisst-cli instance create --json-file ./records/nes/forgotten_in_time_instance.json
../target/debug/gisst-cli instance create --json-file ./records/nes/forgotten_in_time_instance.json
../target/debug/gisst-cli instance create --json-file ./records/nes/forgotten_in_time_instance.json
../target/debug/gisst-cli instance create --json-file ./records/snes/240p_test_suite_SNES_instance.json
../target/debug/gisst-cli instance create --json-file ./records/snes/keeping_SNES_alive_instance.json
../target/debug/gisst-cli instance create --json-file ./records/snes/n_warp_daisakusen_europe_instance.json
../target/debug/gisst-cli instance create --json-file ./records/snes/super_boss_gaiden_japan_instance.json
../target/debug/gisst-cli instance create --json-file ./records/v86/snake_instance.json

# Create NES objects
../target/debug/gisst-cli object create -i --force_uuid $uuid_0 --link $uuid_0 --role content './data/nes/240p Test Suite.nes'
../target/debug/gisst-cli object create -i --force_uuid $uuid_1 --link $uuid_1 --role content './data/nes/Alter Ego.nes'
../target/debug/gisst-cli object create -i --force_uuid $uuid_2 --link $uuid_2 --role content './data/nes/Bobl (v1.1).nes'
../target/debug/gisst-cli object create -i --force_uuid $uuid_3 --link $uuid_3 --role content './data/nes/Chrono Knight.nes'
../target/debug/gisst-cli object create -i --force_uuid $uuid_4 --link $uuid_4 --role content './data/nes/Curse of the Shadow (Prototype).nes'
../target/debug/gisst-cli object create -i --force_uuid $uuid_5 --link $uuid_5 --role content './data/nes/Curse of the Shadow (Prototype).nes'

# Create SNES objects
../target/debug/gisst-cli object create -i --force_uuid $uuid_13 --link $uuid_13 --role content './data/snes/240pSuite.sfc'
../target/debug/gisst-cli object create -i --force_uuid $uuid_14 --link $uuid_14 --role content './data/snes/KeepingSNESalive.sfc'
../target/debug/gisst-cli object create -i --force_uuid $uuid_15 --link $uuid_15 --role content './data/snes/N-Warp Daisakusen (Europe).sfc'
../target/debug/gisst-cli object create -i --force_uuid $uuid_16 --link $uuid_16 --role content './data/snes/Super Boss Gaiden (J) (V1.0).sfc'

# Create v86 objects
../target/debug/gisst-cli object create -i --force_uuid $uuid_21 --link $uuid_21 --role content './data/v86/freedos722.img'

# Create NES config file
../target/debug/gisst-cli object create -i --force_uuid $uuid_6 --link $uuid_0 --role config ./data/nes/retroarch.cfg

# Link NES config object to NES instances
../target/debug/gisst-cli link object $uuid_6 $uuid_1 --role config
../target/debug/gisst-cli link object $uuid_6 $uuid_2 --role config
../target/debug/gisst-cli link object $uuid_6 $uuid_3 --role config
../target/debug/gisst-cli link object $uuid_6 $uuid_4 --role config
../target/debug/gisst-cli link object $uuid_6 $uuid_5 --role config




