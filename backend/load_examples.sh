#!/usr/bin/env zsh

# Some PSX games that support memory cards you can get and put into examples/data/psx:
#  https://archive.org/details/magic-castle-2021-07-may
#  https://nitroyuash.itch.io/petscop-restored

source .env

export GISST_CONFIG_PATH=./config
export MEILI_URL=$MEILI_URL
export MEILI_API_KEY=$MEILI_API_KEY
export MEILI_MASTER_KEY=$MEILI_MASTER_KEY

# To ensure we don't get different results at different times, touch every file to ensure it has the same ATIME/MTIME
find examples/data/ -type f -exec touch -t 202401010101.01 '{}' ';'

# Build most recent GISST Command Line Interface
cargo build --bin gisst-cli

# Initialize search index
./target/debug/gisst-cli init-indices

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
      ./target/debug/gisst-cli add-work-instance --platform-name "Nintendo Entertainment System" --work-version "NTSC" --work-name "$base" \
                               --work-id "$work_uuid" --instance-id "$work_uuid" \
                               --environment-id $uuid_nes_fceumm --cwd examples/data/nes/ \
                               --content "$file"
  elif [ "$folder" = "snes" ]
  then
      ./target/debug/gisst-cli add-work-instance --platform-name "Super Nintendo Entertainment System" --work-version "NTSC" --work-name "$base" \
                               --work-id "$work_uuid" --instance-id "$work_uuid" \
                               --environment-id $uuid_snes9x --cwd examples/data/snes/ \
                               --content "$file"
  elif [ "$folder" = "gb" ]
  then
      ./target/debug/gisst-cli add-work-instance --platform-name "Game Boy" --work-version "NTSC" --work-name "$base" \
                               --work-id "$work_uuid" --instance-id "$work_uuid" \
                               --environment-id $uuid_gambatte --cwd examples/data/gb/ \
                               --content "$file"
      instance_uuid=$(get_uuid_from_counter)
      uuid_counter=$((uuid_counter+1));
      ./target/debug/gisst-cli add-work-instance --platform-name "Game Boy" --work-version "NTSC" --work-name "$base" \
                               --work-id "$work_uuid" --instance-id "$instance_uuid" \
                               --environment-id $uuid_sameboy --cwd examples/data/gb/ \
                               --content "$file"
  elif [ "$folder" = "n64" ]
  then
      ./target/debug/gisst-cli add-work-instance --platform-name "Nintendo 64" --work-version "NTSC" --work-name "$base" \
                               --work-id "$work_uuid" --instance-id "$work_uuid" \
                               --environment-id $uuid_n64 --cwd examples/data/n64/ \
                               --content "$file"
  fi
done

uuid_counter=10000
for work in ./examples/data/psx/*.m3u;
do
  folder=$(basename `dirname "$work"`)
  file=$(basename -- "$work")
  base=${file%.*};
  ext=${file##*.};
  if [ "$base" = "PETSCOP" ]; then
      if [ ${LOAD_PETSCOP:-1} -ne 1 ] ; then
          continue
      fi
  fi
  work_uuid=$(get_uuid_from_counter)
  uuid_counter=$((uuid_counter+1));
  EXTRA_ARGS=
  for DISC_FILE in $(cat "$work"); do
      EXTRA_ARGS="${EXTRA_ARGS} --content ${DISC_FILE}"
      DISC_EXT=${DISC_FILE##*.}
      if [ $DISC_EXT = "cue" ]; then
          for bin in $(awk -F' ' '/^FILE/ {for (i=2; i < NF; i++) { printf "%s", $i; if (i < NF-1) { printf "%s", OFS; } else { printf "\n";}}}' "examples/data/psx/$DISC_FILE"); do
              bin=${bin:1: -1}
              EXTRA_ARGS="${EXTRA_ARGS} --content ${bin}"
          done
      fi
  done
  IFS=' ' read -A extras <<< "$EXTRA_ARGS"
  ./target/debug/gisst-cli add-work-instance --platform-name "Sony Playstation" --work-version "NTSC" --work-name "$base" \
                           --work-id "$work_uuid" --instance-id "$work_uuid" \
                           --environment-id $uuid_pcsx --cwd examples/data/psx/ \
                           --dep scph5500.bin --dep scph5501.bin --dep scph5502.bin \
                           --content "$file" ${extras[@]}
done
for work in examples/data/psx/*.exe; do
  folder=$(basename `dirname "$work"`)
  file=$(basename -- "$work")
  base=${file%.*};
  ext=${file##*.};
  work_uuid=$(get_uuid_from_counter)
  uuid_counter=$((uuid_counter+1));
  ./target/debug/gisst-cli add-work-instance --platform-name "Sony Playstation" --work-version "NTSC" --work-name "$base" \
                           --work-id "$work_uuid" --instance-id "$work_uuid" \
                           --environment-id $uuid_pcsx --cwd examples/data/psx/ \
                           --dep scph5500.bin --dep scph5501.bin --dep scph5502.bin \
                           --content "$file"
done
uuid_psx_1=00000000000000000000000000010000
uuid_psx_2=00000000000000000000000000010001
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

./target/debug/gisst-cli add-work-instance --platform-name "FreeDOS" --work-version "FreeDOS" --work-name "Snake" \
                         --work-id "$work_uuid" --instance-id "$work_uuid" \
                         --environment-id $uuid_v86_freedos --cwd examples/data/v86/ \
                         --content "freedos722.img"

./target/debug/gisst-cli state create --force-uuid "$work_uuid" --link "$work_uuid" --file ./examples/data/v86/snake_state0.v86state --name "Snake Test State" --screenshot-id "$uuid_0" --creator-id "$uuid_0"
./target/debug/gisst-cli replay create --force-uuid "$work_uuid" --link "$work_uuid" --file ./examples/data/v86/snake_replay0.v86replay --name "Snake Test Replay" --creator-id "$uuid_0"

# Clone the snake state
./target/debug/gisst-cli clone-v86 "$work_uuid" "$work_uuid"

work_uuid=$(get_uuid_from_counter)
uuid_counter=$((uuid_counter+1));

if [ ${LOAD_WIN:-1} -eq 1 ] ; then
if [ -f examples/data/v86/win31.img ]; then
./target/debug/gisst-cli add-work-instance --platform-name "Windows 3.1" --work-version "Windows 3.1" --work-name "Windows 3.1" \
                         --work-id "$work_uuid" --instance-id "$work_uuid" \
                         --environment-id $uuid_v86_win_31 --cwd examples/data/v86/ \
                         --content "win31.img"
fi
fi
