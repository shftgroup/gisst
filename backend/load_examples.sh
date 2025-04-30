#!/usr/bin/env zsh

# Some PSX games that support memory cards you can get and put into examples/data/psx:
#  https://archive.org/details/magic-castle-2021-07-may
#  https://nitroyuash.itch.io/petscop-restored

export GISST_CONFIG_PATH=./config

# To ensure we don't get different results at different times, touch every file to ensure it has the same ATIME/MTIME
find examples/data/ -type f -exec touch -t 202401010101.01 '{}' ';'

# Build most recent GISST Command Line Interface
cargo build --bin gisst-cli

# Create default creator
uuid_0=00000000000000000000000000000000
./target/debug/gisst-cli creator create --json-string "{\"creator_id\":\"${uuid_0}\",\"creator_username\":\"GISST\",\"creator_full_name\":\"GISST Test Script\"}"
# Create default screenshot
./target/debug/gisst-cli screenshot create --force-uuid "$uuid_0" ./examples/data/default_screenshot.png

# Create environments for examples
uuid_nes_fceumm=00000000000000000000000000000000
uuid_snes9x=00000000000000000000000000000001
uuid_v86_freedos=00000000000000000000000000000002
uuid_v86_win_31=00000000000000000000000000000003
uuid_n64=00000000000000000000000000000064
uuid_pcsx=00000000000000000000000000000065
uuid_gambatte=00000000000000000000000000000066
uuid_sameboy=00000000000000000000000000000067
./target/debug/gisst-cli environment create --json-file ./examples/records/nes/nes_fceumm_1_52_environment.json
./target/debug/gisst-cli environment create --json-file ./examples/records/snes/snes_snes9x_1_62_3_environment.json
./target/debug/gisst-cli environment create --json-file ./examples/records/v86/freedos_environment.json --environment-config-string '{"bios":{"url":"seabios.bin"},"vga_bios":{"url":"vgabios.bin"},"fda":{"url":"$CONTENT0","async":true,"fixed_chunk_size":44194304}, "memory_size":16777216}'
./target/debug/gisst-cli environment create --json-file ./examples/records/v86/win_31_environment.json --environment-config-string '{"bios":{"url":"seabios.bin"},"vga_bios":{"url":"vgabios.bin"},"memory_size": 67108864, "hda":{"url":"$CONTENT0","async":true,"fixed_chunk_size":44194304}}'
./target/debug/gisst-cli environment create --json-file ./examples/records/n64/n64_gliden64_environment.json
./target/debug/gisst-cli environment create --json-file ./examples/records/psx/psx_pcsx_rearmed_1_62_3_environment.json
./target/debug/gisst-cli environment create --json-file ./examples/records/gb/gambatte_environment.json
./target/debug/gisst-cli environment create --json-file ./examples/records/gb/sameboy_environment.json

# Create retroarch.cfg for nes / snes / n64
uuid_retro_cfg=00000000000000000000000000000100
./target/debug/gisst-cli object create --cwd examples/data/nes --force-uuid $uuid_retro_cfg --role config --role-index 0 retroarch.cfg

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
  elif [ "$uuid_counter" -lt 10000 ]
  then
    echo "0000000000000000000000000000${uuid_counter}"
  elif [ "$uuid_counter" -lt 100000 ]
  then
    echo "000000000000000000000000000${uuid_counter}"
  else
    echo "0000000000000000000000000000${uuid_counter}"
  fi
}

for work in ./examples/data/*/*.{nes,sfc,z64,gb};
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
  elif [ "$folder" = "gb" ]
  then
    ./target/debug/gisst-cli work create --json-string "{\"work_id\":\"$work_uuid\", \"work_name\":\"$base\", \"work_version\":\"NTSC\",\"work_platform\":\"Game Boy\"}"
    ./target/debug/gisst-cli instance create --json-string "{\"instance_id\":\"$work_uuid\", \"environment_id\":\"$uuid_gambatte\", \"work_id\":\"$work_uuid\"}"
    instance_uuid=$(get_uuid_from_counter)
    uuid_counter=$((uuid_counter+1));
    ./target/debug/gisst-cli instance create --json-string "{\"instance_id\":\"$instance_uuid\", \"environment_id\":\"$uuid_sameboy\", \"work_id\":\"$work_uuid\"}"
    ./target/debug/gisst-cli object create --cwd "examples/data/$folder" --force-uuid "$instance_uuid" --link "$instance_uuid" --role content --role-index 0 "$file"
  elif [ "$folder" = "n64" ]
  then
    ./target/debug/gisst-cli work create --json-string "{\"work_id\":\"$work_uuid\", \"work_name\":\"$base\", \"work_version\":\"NTSC\",\"work_platform\":\"Nintendo 64\"}"
    ./target/debug/gisst-cli instance create --json-string "{\"instance_id\":\"$work_uuid\", \"environment_id\":\"$uuid_n64\", \"work_id\":\"$work_uuid\"}"
  fi

  ./target/debug/gisst-cli object create --cwd "examples/data/$folder" --force-uuid "$work_uuid" --link "$work_uuid" --role content --role-index 0 "$file"
done

# add bios files
psx_bios_jp_uuid=$(get_uuid_from_counter)
uuid_counter=$((uuid_counter+1))
./target/debug/gisst-cli object create --role dependency --role-index 0 --force-uuid "$psx_bios_jp_uuid" --cwd examples/data/psx "scph5500.bin"
psx_bios_us_uuid=$(get_uuid_from_counter)
uuid_counter=$((uuid_counter+1))
./target/debug/gisst-cli object create --role dependency --role-index 1 --force-uuid "$psx_bios_us_uuid" --cwd examples/data/psx "scph5501.bin"
psx_bios_eu_uuid=$(get_uuid_from_counter)
uuid_counter=$((uuid_counter+1))
./target/debug/gisst-cli object create --role dependency --role-index 2 --force-uuid "$psx_bios_eu_uuid" --cwd examples/data/psx "scph5502.bin"

uuid_counter=10000
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
  ./target/debug/gisst-cli object create --cwd examples/data/psx --force-uuid "$work_uuid" --link "$work_uuid" --role content --role-index 0 "$file"
  cidx=1
  for DISC_FILE in $(cat "$work"); do
      ./target/debug/gisst-cli object create --cwd examples/data/psx --force-uuid $(get_uuid_from_counter) --link "$work_uuid" --role content --role-index $cidx "$DISC_FILE"
      uuid_counter=$((uuid_counter+1));
      cidx=$((cidx+1))
      DISC_EXT=${DISC_FILE##*.}
      if [ $DISC_EXT = "cue" ]; then
          for bin in $(awk -F' ' '/^FILE/ {for (i=2; i < NF; i++) { printf "%s", $i; if (i < NF-1) { printf "%s", OFS; } else { printf "\n";}}}' "examples/data/psx/$DISC_FILE"); do
              bin=${bin:1: -1}
              ./target/debug/gisst-cli object create --cwd examples/data/psx --force-uuid $(get_uuid_from_counter) --link "$work_uuid" --role content --role-index $cidx "$bin"
              uuid_counter=$((uuid_counter+1));
              cidx=$((cidx+1))
          done
      fi
  done
done
for work in examples/data/psx/*.exe; do
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
  ./target/debug/gisst-cli object create --cwd examples/data/psx --force-uuid "$work_uuid" --link "$work_uuid" --role content --role-index 0 "$file"
done
uuid_psx_1=00000000000000000000000000010000
uuid_psx_2=00000000000000000000000000010003
uuid_save_1=00000000000000000000000000011000
uuid_save_2=00000000000000000000000000011001
uuid_save_3=00000000000000000000000000011002
uuid_save_4=00000000000000000000000000011003
./target/debug/gisst-cli save create --force-uuid "$uuid_save_1" --link "$uuid_psx_1" --file ./examples/data/psx/memcard0a.srm --name "PSX Memory Card 1a" --creator-id "$uuid_0"
./target/debug/gisst-cli save create --force-uuid "$uuid_save_2" --link "$uuid_psx_2" --file ./examples/data/psx/memcard0b.srm --from-save "$uuid_save_1" --name "PSX Memory Card 1b" --creator-id "$uuid_0"
./target/debug/gisst-cli save create --force-uuid "$uuid_save_3" --link "$uuid_psx_1" --file ./examples/data/psx/memcard0c.srm --name "PSX Memory Card 1c" --from-save "$uuid_save_2" --creator-id "$uuid_0"
./target/debug/gisst-cli save create --force-uuid "$uuid_save_4" --link "$uuid_psx_2" --file ./examples/data/psx/memcard1a.srm --name "PSX Memory Card 2" --creator-id "$uuid_0"

uuid_counter=12000
work_uuid=$(get_uuid_from_counter)
uuid_counter=$((uuid_counter+1));

# add a nonsensical ROM hack to the first game in the database
./target/debug/gisst-cli add-patch 00000000000000000000000000001000 examples/records/nes/fake_romhack.json

work_uuid=$(get_uuid_from_counter)
uuid_counter=$((uuid_counter+1));

uuid_counter=12100
work_uuid=$(get_uuid_from_counter)
uuid_counter=$((uuid_counter+1));
# Create v86 objects
./target/debug/gisst-cli work create --json-string "{\"work_id\":\"$work_uuid\", \"work_name\":\"Snake\", \"work_version\":\"FreeDOS\",\"work_platform\":\"FreeDOS\"}"
./target/debug/gisst-cli instance create --json-string "{\"instance_id\":\"$work_uuid\", \"environment_id\":\"$uuid_v86_freedos\", \"work_id\":\"$work_uuid\"}"
./target/debug/gisst-cli object create --cwd examples/data/v86 --force-uuid "$work_uuid" --link "$work_uuid" --role content --role-index 0 'freedos722.img'
./target/debug/gisst-cli state create --force-uuid "$work_uuid" --link "$work_uuid" --file ./examples/data/v86/snake_state0.v86state --name "Snake Test State" --screenshot-id "$uuid_0" --creator-id "$uuid_0"
./target/debug/gisst-cli replay create --force-uuid "$work_uuid" --link "$work_uuid" --file ./examples/data/v86/snake_replay0.v86replay --name "Snake Test Replay" --creator-id "$uuid_0"

# Clone the snake state
./target/debug/gisst-cli clone-v86 "$work_uuid" "$work_uuid"

work_uuid=$(get_uuid_from_counter)
uuid_counter=$((uuid_counter+1));

if [ ${LOAD_WIN:-1} -eq 1 ] ; then
if [ -f examples/data/v86/win31.img ]; then
  ./target/debug/gisst-cli work create --json-string "{\"work_id\":\"$work_uuid\", \"work_name\":\"Window 3.1\", \"work_version\":\"Windows 3.1\", \"work_platform\":\"Windows 3.1\"}"
  ./target/debug/gisst-cli instance create --json-string "{\"instance_id\":\"$work_uuid\", \"environment_id\":\"$uuid_v86_win_31\", \"work_id\":\"$work_uuid\"}"
  ./target/debug/gisst-cli object create --cwd examples/data/v86 --force-uuid "$work_uuid" --link "$work_uuid" --role content --role-index 0 'win31.img'
fi
fi
