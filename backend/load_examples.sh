#!/usr/bin/env zsh

export GISST_CONFIG_PATH=./config
# Build most recent GISST Command Line Interface
cargo build --bin gisst-cli

# Create environments for examples
uuid_nes_fceumm=00000000000000000000000000000000
uuid_snes9x=00000000000000000000000000000001
uuid_v86_freedos=00000000000000000000000000000002
uuid_v86_win_31=00000000000000000000000000000003
uuid_n64=00000000000000000000000000000064
uuid_pcsx=00000000000000000000000000000065
./target/debug/gisst-cli environment create --json-file ./examples/records/nes/nes_fceumm_1_52_environment.json
./target/debug/gisst-cli environment create --json-file ./examples/records/snes/snes_snes9x_1_62_3_environment.json
./target/debug/gisst-cli environment create --json-file ./examples/records/v86/freedos_environment.json --environment-config-string '{"bios":{"url":"seabios.bin"},"vga_bios":{"url":"vgabios.bin"},"fda":{"url":"$CONTENT0","async":true,"fixed_chunk_size":44194304}, "memory_size":16777216}'
./target/debug/gisst-cli environment create --json-file ./examples/records/v86/win_31_environment.json --environment-config-string '{"bios":{"url":"seabios.bin"},"vga_bios":{"url":"vgabios.bin"},"memory_size": 67108864, "hda":{"url":"$CONTENT0","async":true,"fixed_chunk_size":44194304}}'
./target/debug/gisst-cli environment create --json-file ./examples/records/n64/n64_gliden64_environment.json
./target/debug/gisst-cli environment create --json-file ./examples/records/psx/psx_pcsx_rearmed_1_62_3_environment.json

# Create retroarch.cfg for nes / snes / n64
uuid_retro_cfg=00000000000000000000000000000100
./target/debug/gisst-cli object create -i --cwd examples/data/nes --force-uuid $uuid_retro_cfg --role config --role-index 0 retroarch.cfg

# Set up counter for work / instance UUID
uuid_counter=1000

get_uuid_from_counter() {
  if [ "$uuid_counter" -lt 10 ]
  then
    echo "0000000000000000000000000000000${uuid_counter}"
  elif [ "$uuid_counter" -lt 100 ]
  then
    echo "000000000000000000000000000000${uuid_counter}"
  elif [ "$uuid_counter" -lt 1000 ]
  then
    echo "00000000000000000000000000000${uuid_counter}"
  else
    echo "0000000000000000000000000000${uuid_counter}"
  fi
}

for work in ./examples/data/*/*.{nes,sfc,z64};
do
  folder=$(basename `dirname "$work"`)
  file=$(basename -- "$work")
  base=${file%.*};
  ext=${file##*.};
  work_uuid=$(get_uuid_from_counter)
  uuid_counter=$((uuid_counter+1));


  if [ "$folder" = "nes" ]
  then
    ./target/debug/gisst-cli work create --json-string "{\"work_id\":\"$work_uuid\", \"work_name\":\"$base\", \"work_version\":\"NTSC\",\"work_platform\":\"Nintendo Entertainment System\"}"
    ./target/debug/gisst-cli instance create --json-string "{\"instance_id\":\"$work_uuid\", \"environment_id\":\"$uuid_nes_fceumm\", \"work_id\":\"$work_uuid\"}"
    ./target/debug/gisst-cli link object $uuid_retro_cfg $work_uuid --role config
  elif [ "$folder" = "snes" ]
  then
    ./target/debug/gisst-cli work create --json-string "{\"work_id\":\"$work_uuid\", \"work_name\":\"$base\", \"work_version\":\"NTSC\",\"work_platform\":\"Super Nintendo Entertainment System\"}"
    ./target/debug/gisst-cli instance create --json-string "{\"instance_id\":\"$work_uuid\", \"environment_id\":\"$uuid_snes9x\", \"work_id\":\"$work_uuid\"}"
  elif [ "$folder" = "n64" ]
  then
    ./target/debug/gisst-cli work create --json-string "{\"work_id\":\"$work_uuid\", \"work_name\":\"$base\", \"work_version\":\"NTSC\",\"work_platform\":\"Nintendo 64\"}"
    ./target/debug/gisst-cli instance create --json-string "{\"instance_id\":\"$work_uuid\", \"environment_id\":\"$uuid_n64\", \"work_id\":\"$work_uuid\"}"
  fi

  ./target/debug/gisst-cli object create --cwd "examples/data/$folder" -i --force-uuid "$work_uuid" --link "$work_uuid" --role content --role-index 0 "$file"
done

# add bios files
psx_bios_jp_uuid=$(get_uuid_from_counter)
uuid_counter=$((uuid_counter+1))
./target/debug/gisst-cli object create -i --role dependency --role-index 0 --force-uuid "$psx_bios_jp_uuid" --cwd examples/data/psx "scph5500.bin"
psx_bios_us_uuid=$(get_uuid_from_counter)
uuid_counter=$((uuid_counter+1))
./target/debug/gisst-cli object create -i --role dependency --role-index 1 --force-uuid "$psx_bios_us_uuid" --cwd examples/data/psx "scph5501.bin"
psx_bios_eu_uuid=$(get_uuid_from_counter)
uuid_counter=$((uuid_counter+1))
./target/debug/gisst-cli object create -i --role dependency --role-index 2 --force-uuid "$psx_bios_eu_uuid" --cwd examples/data/psx "scph5502.bin"

for work in ./examples/data/psx/*.m3u;
do
  folder=$(basename `dirname "$work"`)
  file=$(basename -- "$work")
  base=${file%.*};
  ext=${file##*.};
  work_uuid=$(get_uuid_from_counter)
  uuid_counter=$((uuid_counter+1));
  ./target/debug/gisst-cli work create --json-string "{\"work_id\":\"$work_uuid\", \"work_name\":\"$base\", \"work_version\":\"NTSC\",\"work_platform\":\"Sony Playstation\"}"
  ./target/debug/gisst-cli instance create --json-string "{\"instance_id\":\"$work_uuid\", \"environment_id\":\"$uuid_pcsx\", \"work_id\":\"$work_uuid\"}"
  ./target/debug/gisst-cli link object $uuid_retro_cfg $work_uuid --role config
  ./target/debug/gisst-cli link object $psx_bios_jp_uuid $work_uuid --role dependency --role-index 0
  ./target/debug/gisst-cli link object $psx_bios_us_uuid $work_uuid --role dependency --role-index 1
  ./target/debug/gisst-cli link object $psx_bios_eu_uuid $work_uuid --role dependency --role-index 2
  ./target/debug/gisst-cli object create -i --cwd examples/data/psx --force-uuid "$work_uuid" --link "$work_uuid" --role content --role-index 0 "$file"
  cidx=1
  for DISC_FILE in $(cat "$work"); do
      ./target/debug/gisst-cli object create -i --cwd examples/data/psx --force-uuid $(get_uuid_from_counter) --link "$work_uuid" --role content --role-index $cidx "$DISC_FILE"
      uuid_counter=$((uuid_counter+1));
      cidx=$((cidx+1))
      DISC_EXT=${DISC_FILE##*.}
      if [ $DISC_EXT = "cue" ]; then
          for bin in $(awk -F' ' '/^FILE/ {for (i=2; i < NF; i++) { printf "%s", $i; if (i < NF-1) { printf "%s", OFS; } else { printf "\n";}}}' "examples/data/psx/$DISC_FILE"); do
              bin=${bin:1: -1}
              ./target/debug/gisst-cli object create -i --cwd examples/data/psx --force-uuid $(get_uuid_from_counter) --link "$work_uuid" --role content --role-index $cidx "$bin"
              uuid_counter=$((uuid_counter+1));
              cidx=$((cidx+1))
          done
      fi
  done
done


# Create default creator
uuid_0=00000000000000000000000000000000
./target/debug/gisst-cli creator create --json-string "{\"creator_id\":\"${uuid_0}\",\"creator_username\":\"GISST\",\"creator_full_name\":\"GISST Test Script\"}"
# Create default screenshot
./target/debug/gisst-cli screenshot create --force-uuid "$uuid_0" ./examples/data/default_screenshot.png

work_uuid=$(get_uuid_from_counter)
uuid_counter=$((uuid_counter+1));
# Create v86 objects
./target/debug/gisst-cli work create --json-string "{\"work_id\":\"$work_uuid\", \"work_name\":\"Snake\", \"work_version\":\"FreeDOS\",\"work_platform\":\"FreeDOS\"}"
./target/debug/gisst-cli instance create --json-string "{\"instance_id\":\"$work_uuid\", \"environment_id\":\"$uuid_v86_freedos\", \"work_id\":\"$work_uuid\"}"
./target/debug/gisst-cli object create -i --cwd examples/data/v86 --force-uuid "$work_uuid" --link "$work_uuid" --role content --role-index 0 'freedos722.img'
./target/debug/gisst-cli state create --force-uuid "$work_uuid" --link "$work_uuid" --file ./examples/data/v86/snake_state0.v86state --name "Snake Test State" --screenshot-id "$uuid_0" --creator-id "$uuid_0"
./target/debug/gisst-cli replay create --force-uuid "$work_uuid" --link "$work_uuid" --file ./examples/data/v86/snake_replay0.v86replay --name "Snake Test Replay" --creator-id "$uuid_0"
work_uuid=$(get_uuid_from_counter)
uuid_counter=$((uuid_counter+1));
./target/debug/gisst-cli work create --json-string "{\"work_id\":\"$work_uuid\", \"work_name\":\"Window 3.1\", \"work_version\":\"Windows 3.1\", \"work_platform\":\"Windows 3.1\"}"
./target/debug/gisst-cli instance create --json-string "{\"instance_id\":\"$work_uuid\", \"environment_id\":\"$uuid_v86_win_31\", \"work_id\":\"$work_uuid\"}"
./target/debug/gisst-cli object create -i --cwd examples/data/v86 --force-uuid "$work_uuid" --link "$work_uuid" --role content --role-index 0 'win31.img'