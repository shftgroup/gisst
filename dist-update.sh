#!/bin/bash

rm -rf retroarch-web
curl -o /tmp/retro.7z https://buildbot.libretro.com/nightly/emscripten/RetroArch.7z
7z x /tmp/retro.7z -o.
mv retroarch retroarch-web
mkdir -p frontend-web/public/cores
cp retroarch-web/*.{js,wasm} frontend-web/public/cores/
mkdir -p frontend-web/public/assets
cp -r retroarch-web/assets/frontend frontend-web/public/assets/

# trim down the bundle size some
cd frontend-web/public/assets/frontend/bundle
rm -rf overlays shaders filters assets/glui assets/xmb assets/rgui
../../../../../retroarch-web/indexer . > .index-xhr
cd -
