#!/bin/bash

set -e

# PLATFORM_MAIN may be one of:
# linux
# osx
# windows
# PLATFORM_SUB may be one of:
# linux/windows:
#   x86
#   x86_64
# osx:
#   x86_64
#   arm64

PLATFORM_MAIN="${1:-linux}"
PLATFORM_SUB="${2:-x86_64}"

rm -rf retroarch-web
curl -o /tmp/retro.7z https://buildbot.libretro.com/nightly/emscripten/RetroArch.7z
7z x /tmp/retro.7z -o.
mv retroarch retroarch-web
mkdir -p frontend-web/public/cores
cp retroarch-web/*.{js,wasm} frontend-web/public/cores/
mkdir -p frontend-web/public/assets
rm -rf frontend-web/public/assets/frontend
cp -r retroarch-web/assets/frontend frontend-web/public/assets/

cd gisst-player-ui
npm i
cd ..

# trim down the bundle size some
cd frontend-web
npm i
cd ..

cd frontend-elec
npm i
cd ..

cd frontend-web/public/assets/frontend/bundle
rm -rf overlays shaders filters database assets/glui assets/xmb assets/rgui
../../../../node_modules/coffeescript/bin/coffee ../../../../../retroarch-web/indexer . > .index-xhr
cd -

rm -rf frontend-tauri/src-tauri/ra-assets
cp -r frontend-web/public/assets/frontend/bundle frontend-tauri/src-tauri/ra-assets

rm -rf frontend-elec/resources/ra-assets/
mkdir -p frontend-elec/resources
cp -r frontend-web/public/assets/frontend/bundle frontend-elec/resources/ra-assets

case $PLATFORM_MAIN in
    osx)
        curl -o /tmp/retroarch.dmg https://buildbot.libretro.com/nightly/apple/osx/universal/RetroArch_Metal.dmg
        hdiutil attach /tmp/retroarch.dmg
        cp -r /Volumes/RetroArch/RetroArch.app frontend-elec/resources/binaries/
        # TODO it would be nice to unmount the dmg
        curl "https://buildbot.libretro.com/nightly/apple/osx/$PLATFORM_SUB/latest/" | tr ">" "\n" | sed -Ern 's|<a href="/([a-zA-Z0-9_]+/)+([a-zA-Z0-9_]+.dylib.zip)"|\2|p;' > /tmp/cores.txt
        mkdir -p /tmp/cores
        mkdir -p frontend-elec/resources/ra-assets/cores
        while read core; do
            curl -o /tmp/cores/$core "https://buildbot.libretro.com/nightly/apple/osx/arm64/latest/$core"
            unzip "/tmp/cores/$core" -d frontend-elec/resources/ra-assets/cores/
        done < /tmp/cores.txt
    ;;
    windows)
        echo "this should probably be a separate batchfile"
    ;;
    linux)
        curl -o /tmp/retroarch_linux.7z "https://buildbot.libretro.com/nightly/linux/$PLATFORM_SUB/RetroArch.7z"
        curl -o /tmp/retroarch_linux_cores.7z "https://buildbot.libretro.com/nightly/linux/$PLATFORM_SUB/RetroArch_cores.7z"
        mkdir -p /tmp/ralin
        7z x /tmp/retroarch_linux.7z -o/tmp/ralin
        cp "/tmp/ralin/RetroArch-Linux-$PLATFORM_SUB/RetroArch-Linux-$PLATFORM_SUB.AppImage" frontend-elec/resources/binaries/RetroArch.AppImage
        mkdir -p frontend-elec/resources/ra-assets/cores
        7z x /tmp/retroarch_linux_cores.7z -o/tmp/ralin
        cp /tmp/ralin/"RetroArch-Linux-$PLATFORM_SUB/RetroArch-Linux-$PLATFORM_SUB.AppImage.home/.config/retroarch/cores/"* frontend-elec/resources/ra-assets/cores/
    ;;
esac
