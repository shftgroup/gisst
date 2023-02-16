#!/bin/bash

rm -rf retroarch-web
curl -o /tmp/retro.7z https://buildbot.libretro.com/nightly/emscripten/RetroArch.7z
7z x /tmp/retro.7z -o.
mv retroarch retroarch-web
mkdir -p frontend-web/public/cores
cp retroarch-web/*.{js,wasm} frontend-web/public/cores/
mkdir -p frontend-web/public/assets
rm -rf frontend-web/public/assets/frontend
cp -r retroarch-web/assets/frontend frontend-web/public/assets/

# trim down the bundle size some
cd frontend-web/public/assets/frontend/bundle
rm -rf overlays shaders filters database assets/glui assets/xmb assets/rgui
../../../../../retroarch-web/indexer . > .index-xhr
cd -

rm -rf frontend-tauri/src-tauri/ra-assets
cp -r frontend-web/public/assets/frontend/bundle frontend-tauri/src-tauri/ra-assets

rm -rf frontend-elec/resources
cp -r frontend-web/public/assets/frontend/bundle frontend-elec/resources/ra-assets
