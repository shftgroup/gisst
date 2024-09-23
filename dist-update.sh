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

