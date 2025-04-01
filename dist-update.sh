#!/bin/bash

set -e

function die
{
    local message=$1
    [ -z "$message" ] && message="Died"
    echo "${BASH_SOURCE[1]}: line ${BASH_LINENO[0]}: ${FUNCNAME[1]}: $message." >&2
    exit 1
}

mkdir -p frontend/frontend-web/public/cores
mkdir -p frontend/frontend-web/public/assets

if [ ${GET_ASSETS:-1} -eq 1 ] ; then
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
rm -rf overlays/{borders,ctr,effects,ipad,misc,wii}
rm -rf overlays/gamepads/{720-med,arcade,arcade-anim,arcade-minimal,cdi_anim_portrait,example,flat,flip_phone,gba-anim_landscape,gba-grey,gb_anim_portrait,genesis,lite,n64,old,rgpad,scummvm}
rm -f assets/ozone/png/icons/*\ -\ *
rm -f assets/sounds/*.wav
rm -f assets/pkg/chinese-*
rm -f assets/pkg/korean-*
rm -rf assets/pkg/wiiu
rm -rf assets/{xmb,switch,rgui,glui}
zip -r -l ../assets_minimal.zip *
cp ../assets_minimal.zip ../frontend/frontend-web/public/assets/frontend/
popd
fi

if [ ${FETCH_CORES:-1} -eq 1 ] ; then
mkdir -p ra-build
pushd ra-build
mkdir -p cores
emsdk install 4.0.6
emsdk activate 4.0.6
git clone --depth 1 -b tick-event https://github.com/JoeOsborn/v86 v86 || echo "already have v86"
git clone --depth 1 https://github.com/libretro/retroarch ra || echo "already have RA"
git clone --depth 1 https://github.com/libretro/libretro-fceumm fceumm || echo "already have fceumm"
git clone --depth 1 https://github.com/libretro/snes9x snes9x || echo "already have snes9x"
git clone --depth 1 -b emscripten-build https://github.com/JoeOsborn/hatari hatari || echo "already have hatari"
git clone --depth 1 https://github.com/libretro/stella2014-libretro stella2014 || echo "already have stella2014"
git clone --depth 1 -b emscripten-build-fixes https://github.com/JoeOsborn/pcsx_rearmed pcsx_rearmed || echo "already have pcsx"
git clone --depth 1 -b fix-makefile-emscripten https://github.com/JoeOsborn/vba-next vba_next || echo "already have vba"
git clone --depth 1 https://github.com/libretro/gambatte-libretro gambatte || echo "already have gambatte"
git clone --depth 1 https://github.com/LIJI32/SameBoy sameboy || echo "already have sameboy"
git clone --depth 1 -b emscripten_testing https://github.com/JoeOsborn/mupen64plus-libretro-nx mupen64plus_next || echo "already have mupen64"
popd
fi

if [ ${BUILD_CORES:-1} -eq 1 ]; then
pushd ra-build

pushd ra
rm -rf obj-emscripten
git pull || echo "RA directory dirty or pull failed for other reason"
popd

for f in *; do
    if [ $f = "ra" ] || [ $f = "cores" ]; then
        continue
    fi
    pushd $f
    git pull || echo "${f} pull failed"

    ASYNC=0
    if [ $f = "mupen64plus_next" ] || [ $f = "hatari" ]; then ASYNC=1; fi

    if [ $f = "v86" ]
    then
        # make clean
        WASM_OPT=true PATH="${PATH}:${EMSDK}/upstream/bin" make all -j || die "could not build v86"
        cp build/libv86.js build/v86.wasm ../../frontend/frontend-web/public/v86
        popd
        continue
    elif [ $f = "sameboy" ]
    then
        git clone --depth 1 git@github.com:gbdev/rgbds.git || echo "Could not get rgbds or rgbds already present"
        make -C rgbds -j || die "Could not build rgbds"
        PATH="./rgbds:${PATH}" make -j CONF=release bootroms || die "could not build sameboy bootroms"
        PATH="./rgbds:${PATH}" emmake make CONF=release platform=emscripten pthread=4 STATIC_LINKING=1 ASYNC=$ASYNC -j libretro || die "could not build core ${f}"
        cp build/bin/${f}_libretro_emscripten.bc ../ra/libretro_emscripten.bc
    elif [ -f Makefile.libretro ]
    then
        # emmake make -f Makefile.libretro platform=emscripten clean
        emmake make -f Makefile.libretro platform=emscripten pthread=4 STATIC_LINKING=1 ASYNC=$ASYNC -j || die "could not build core ${f}"
        cp ${f}_libretro_emscripten.bc ../ra/libretro_emscripten.bc
    elif [ -d libretro ] && [ -f libretro/Makefile ]
    then
        pushd libretro
        # emmake make platform=emscripten clean
        emmake make platform=emscripten pthread=4 STATIC_LINKING=1 ASYNC=$ASYNC -j || die "could not build core ${f}"
        cp ${f}_libretro_emscripten.bc ../../ra/libretro_emscripten.bc
        popd
    else
        # emmake make platform=emscripten clean
        emmake make platform=emscripten pthread=4 STATIC_LINKING=1 ASYNC=$ASYNC -j || die "could not build core ${f}"
        cp ${f}_libretro_emscripten.bc ../ra/libretro_emscripten.bc
    fi
    pushd ../ra
    cp libretro_emscripten.bc libretro_emscripten.a
    emmake make -f Makefile.emscripten LIBRETRO=$f HAVE_THREADS=1 PTHREAD_POOL_SIZE=4 PROXY_TO_PTHREAD=1 HAVE_WASMFS=1 HAVE_EGL=0 ASYNC=$ASYNC SYMBOLS=1 HAVE_OPENGLES3=1 -j all || die "could not build RA dist for ${f}"
    cp ${f}_libretro.* ../cores
    popd
    popd
done
popd
cp ra-build/cores/* frontend/frontend-web/public/cores/
fi

if [ ${BUILD_FRONTEND:-1} -eq 1 ]; then
cd frontend
npm i
npm run build --workspaces --if-present
npm run dist --workspaces --if-present
cd ..
fi
