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
mkdir -p frontend/frontend-web/public/cores
cp retroarch-web/*.{js,wasm} frontend/frontend-web/public/cores/
mkdir -p frontend/frontend-web/public/assets
rm -rf frontend/frontend-web/public/assets/frontend
cp -r retroarch-web/assets/frontend frontend/frontend-web/public/assets/

pushd frontend/frontend-web/public/assets/frontend
rm -f bundle.zip
cat bundle.* > bundle-ra.zip
rm bundle.zip.*
unzip bundle-ra.zip
rm bundle-ra.zip
cd bundle
rm -rf shaders filters database assets/glui assets/xmb assets/rgui
rm -rf overlays/{borders,ctr,effects,ipad,keyboards,misc,wii}
rm -rf overlays/gamepads/{720-med,arcade,arcade-anim,arcade-minimal,cdi_anim_portrait,dual-shock,example,flat,flip_phone,gba-anim_landscape,gba-grey,gb_anim_portrait,genesis,lite,n64,neo-retropad,old,psx,rgpad,scummvm}
rm -f assets/ozone/png/icons/*\ -\ *
rm -f assets/sounds/*.wav
rm -f assets/pkg/chinese-*
rm -f assets/pkg/korean-*
cd ..
zip -r -1 bundle.zip bundle
rm -rf bundle
popd

cd frontend
npm i
cd ..

rm -rf frontend/frontend-elec/resources/ra-assets/
mkdir -p frontend/frontend-elec/resources
cp -r frontend/frontend-web/public/assets/frontend/bundle frontend/frontend-elec/resources/ra-assets

case $PLATFORM_MAIN in
    osx)
        curl -o /tmp/retroarch.dmg https://buildbot.libretro.com/nightly/apple/osx/universal/RetroArch_Metal.dmg
        hdiutil attach /tmp/retroarch.dmg
        cp -r /Volumes/RetroArch/RetroArch.app frontend/frontend-elec/resources/binaries/
        # TODO it would be nice to unmount the dmg
        curl "https://buildbot.libretro.com/nightly/apple/osx/$PLATFORM_SUB/latest/" | tr ">" "\n" | sed -Ern 's|<a href="/([a-zA-Z0-9_]+/)+([a-zA-Z0-9_]+.dylib.zip)"|\2|p;' > /tmp/cores.txt
        mkdir -p /tmp/cores
        mkdir -p frontend/frontend-elec/resources/ra-assets/cores
        while read core; do
            curl -o /tmp/cores/$core "https://buildbot.libretro.com/nightly/apple/osx/arm64/latest/$core"
            unzip "/tmp/cores/$core" -d frontend/frontend-elec/resources/ra-assets/cores/
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
        mkdir -p frontend/frontend-elec/resources/binaries
        cp "/tmp/ralin/RetroArch-Linux-$PLATFORM_SUB/RetroArch-Linux-$PLATFORM_SUB.AppImage" frontend/frontend-elec/resources/binaries/RetroArch.AppImage
        mkdir -p frontend/frontend-elec/resources/ra-assets/cores
        7z x /tmp/retroarch_linux_cores.7z -o/tmp/ralin
        cp /tmp/ralin/"RetroArch-Linux-$PLATFORM_SUB/RetroArch-Linux-$PLATFORM_SUB.AppImage.home/.config/retroarch/cores/"* frontend/frontend-elec/resources/ra-assets/cores/
    ;;
esac
