#!/bin/zsh

pushd "$1"
IFS=$'\n'
for cue in $(find * -type f \( -iname "*.cue" -o -iname "*.chd" -o -iname "*.iso" \) | sed -e 's/ (Disc\ [0-9].*//' -e 's/\.cue//' | uniq); do
    if [ ! -f ${cue}.m3u ]; then
        find * -type f -name "$cue*" \( -iname "*.cue" -o -iname "*.chd" -o -iname "*.iso" \) > ${cue}.m3u
    fi
done

popd
