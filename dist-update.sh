#!/bin/bash

set -e

function die
{
    local message=$1
    [ -z "$message" ] && message="Died"
    echo "${BASH_SOURCE[1]}: line ${BASH_LINENO[0]}: ${FUNCNAME[1]}: $message." >&2
    exit 1
}

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

mkdir -p frontend/frontend-web/public/cores
mkdir -p frontend/frontend-web/public/assets

rm -rf retroarch-web
mkdir -p assets
pushd assets
rm -f ../assets_minimal.zip
rm -rf assets overlays info
curl -o assets.zip https://buildbot.libretro.com/assets/frontend/assets.zip
curl -o overlays.zip https://buildbot.libretro.com/assets/frontend/overlays.zip
curl -o info.zip https://buildbot.libretro.com/assets/frontend/info.zip
unzip assets.zip -d assets
rm assets.zip
unzip overlays.zip -d overlays
rm overlays.zip
unzip info.zip -d info
rm info.zip
rm -rf overlays/{borders,ctr,effects,ipad,keyboards,misc,wii}
rm -rf overlays/gamepads/{720-med,arcade,arcade-anim,arcade-minimal,cdi_anim_portrait,dual-shock,example,flat,flip_phone,gba-anim_landscape,gba-grey,gb_anim_portrait,genesis,lite,n64,neo-retropad,old,psx,rgpad,scummvm}
rm -f assets/ozone/png/icons/*\ -\ *
rm -f assets/sounds/*.wav
rm -f assets/pkg/chinese-*
rm -f assets/pkg/korean-*
rm -rf assets/pkg/wiiu
rm -rf assets/{xmb,switch,rgui,glui}
zip -r -l ../assets_minimal.zip *
cp ../assets_minimal.zip ../frontend/frontend-web/public/assets/frontend/
popd

mkdir -p ra-build
pushd ra-build
mkdir -p cores
emsdk install tot
emsdk activate tot
git clone --depth 1 https://github.com/libretro/retroarch ra || echo "already have RA"
git clone --depth 1 https://github.com/libretro/libretro-fceumm fceumm || echo "already have fceumm"
git clone --depth 1 https://github.com/libretro/snes9x snes9x || echo "already have snes9x"
git clone --depth 1 https://github.com/libretro/pcsx_rearmed pcsx_rearmed || echo "already have pcsx"
git clone --depth 1 https://github.com/libretro/vba-next vba_next || echo "already have vba"
git clone --depth 1 https://github.com/libretro/gambatte-libretro gambatte || echo "already have gambatte"
git clone --depth 1 https://github.com/libretro/mupen64plus-libretro-nx mupen64plus_next || echo "already have mupen64"


cd ra
rm -rf ra/obj-emscripten
emconfigure ./configure
cd ..

for f in {fceumm,snes9x,pcsx_rearmed,vba_next,gambatte}; do
    pushd $f
    git pull
    if [ -f Makefile.libretro ]
    then
        # emmake make -f Makefile.libretro platform=emscripten clean
        emmake make -f Makefile.libretro platform=emscripten pthread=4 STATIC_LINKING=1 -j || die "could not build core ${f}"
        cp ${f}_libretro_emscripten.bc ../ra/libretro_emscripten.bc
    elif [ -d libretro ] && [ -f libretro/Makefile ]
    then
        pushd libretro
        # emmake make platform=emscripten clean
        emmake make platform=emscripten pthread=4 STATIC_LINKING=1 -j || die "could not build core ${f}"
        cp ${f}_libretro_emscripten.bc ../../ra/libretro_emscripten.bc
        popd
    else
        # emmake make platform=emscripten clean
        emmake make platform=emscripten pthread=4 STATIC_LINKING=1 -j || die "could not build core ${f}"
        cp ${f}_libretro_emscripten.bc ../ra/libretro_emscripten.bc
    fi
    pushd ../ra
    cp libretro_emscripten.bc libretro_emscripten.a
    emmake make -f Makefile.emscripten LIBRETRO=$f PTHREAD=4 PROXY_TO_PTHREAD=1 HAVE_WASMFS=1 HAVE_EGL=0 ASYNC=0 -j all || die "could not build RA dist for ${f}"
    cp ${f}_libretro.* ../cores
    popd
    popd
done
popd

cp ra-build/cores/* frontend/frontend-web/public/cores/

cd frontend
npm i
npm run build --workspaces --if-present
npm run dist --workspaces --if-present
cd ..

