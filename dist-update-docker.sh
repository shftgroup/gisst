#!/bin/bash

set -e

git config --global advice.detachedHead false

function die
{
    local message=$1
    [ -z "$message" ] && message="Died"
    echo "${BASH_SOURCE[1]}: line ${BASH_LINENO[0]}: ${FUNCNAME[1]}: $message." >&2
    exit 1

}

[ -e /manifest.json ] || die "No manifest file available in container at /manifest.json"

function getrepo
{
    local dir=$1
    local jq_prefix=$2
    local repo=$(jq -re "${jq_prefix}.repo" /manifest.json) || die "Could not get repository url"
    local vsn=$(jq -re "${jq_prefix}.version" /manifest.json) || die "Could not get repository version"
    if [ -e $dir ]; then
        cd $dir
        if [ $vsn != $(git rev-parse HEAD) ]; then
            echo "Checkout of $dir exists with wrong version $(git rev-parse HEAD) (expected $vsn); removing..."
            rm -r $dir
        else
            echo "$dir already checked out at correct version"
            return 0;
        fi
    fi
    git clone --depth 1 --revision $vsn $repo $dir
    return $?
}

# check rust and emsdk versions from env var against manifest
MANIFEST_RUST_VERSION=$(jq -r ".rust_version" /manifest.json)
[ $MANIFEST_RUST_VERSION = "${RUST_VERSION}" ] || die "Container built with wrong rust version ${RUST_VERSION} for manifest $MANIFEST_RUST_VERSION"
MANIFEST_EMSDK_VERSION=$(jq -r ".emsdk_version" /manifest.json)
[ $MANIFEST_EMSDK_VERSION = "${EMSDK_VERSION}" ] || die "Container built with wrong emsdk version ${EMSDK_VERSION} for manifest $MANIFEST_EMSDK_VERSION"

mkdir -p /out/cores

# Make asset bundle

if [ ${GET_ASSETS:-1} -eq 1 ] ; then
rm -rf retroarch-web
mkdir -p assets
pushd assets
rm -f ../assets_minimal.zip
rm -rf assets overlays info
curl -o assets.zip https://buildbot.libretro.com/assets/frontend/assets.zip
curl -o overlays.zip https://buildbot.libretro.com/assets/frontend/overlays.zip
unzip assets.zip -d assets
rm assets.zip
unzip overlays.zip -d overlays
rm overlays.zip
rm -rf overlays/{configure,borders,ctr,effects,ipad,Makefile,misc,wii}
rm -rf overlays/gamepads/{720-med,arcade,arcade-anim,arcade-minimal,cdi_anim_portrait,example,flip_phone,gba-anim_landscape,gba-grey,gb_anim_portrait,genesis,lite,n64,neo-ds-portrait,nes-small,old,Piixel-Gamepads,quadpad,rgpad,scummvm}
rm -rf overlays/keyboards/{commodore,modular-keyboard}
rm -f assets/sounds/*.wav
rm -f assets/sounds/bgm.ogg
rm -f assets/pkg/chinese-*
rm -f assets/pkg/korean-*
rm -rf assets/pkg/wiiu
rm -rf assets/{xmb,switch,glui,nxrgui,ozone}
rm -f assets/rgui/*.cfg
rm -rf assets/rgui/wallpaper
zip -r -l ../assets_minimal.zip *
cp ../assets_minimal.zip /out/assets_minimal.zip
popd
fi

if [ ${FETCH_CORES:-1} -eq 1 ] ; then
mkdir -p ra-build
pushd ra-build

getrepo v86 '.v86'
getrepo ra '.retroarch'

CORENAMES=$(jq -re '.retroarch_cores | to_entries[] | select(.value.skip|not) | .key' /manifest.json)

for corename in $CORENAMES; do
    getrepo $corename ".retroarch_cores.${corename}"
done

popd
fi

if [ ${BUILD_CORES:-1} -eq 1 ]; then
pushd ra-build

pushd ra
rm -rf obj-emscripten
popd

for f in $CORENAMES v86; do
    if [ $f = "ra" ]; then
        continue
    fi
    pushd $f

    ASYNC=0
    # TODO get from manifest
    if [ $f = "mupen64plus_next" ] || [ $f = "hatari" ]; then ASYNC=1; fi

    if [ $f = "v86" ]
    then
        # make clean
        WASM_OPT=true PATH="${PATH}:${EMSDK}/upstream/bin" make all -j || die "could not build v86"
        cp build/libv86.js build/v86.wasm /out/
        popd
        continue
    elif [ $f = "sameboy" ]
    then
        git clone --depth 1 https://github.com/gbdev/rgbds.git || echo "Could not get rgbds or rgbds already present"
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
    emmake make -f Makefile.emscripten LIBRETRO=$f HAVE_RWEBAUDIO=0 HAVE_THREADS=1 PTHREAD_POOL_SIZE=4 PROXY_TO_PTHREAD=1 HAVE_WASMFS=1 HAVE_EXTRA_WASMFS=1 HAVE_EGL=0 HAVE_AL=0 HAVE_AUDIOWORKLET=1 ASYNC=$ASYNC SYMBOLS=0 HAVE_OPENGLES3=1 HAVE_OZONE=0 HAVE_XMB=0 HAVE_GLUI=0 HAVE_MATERIALUI=0 -j all || die "could not build RA dist for ${f}"
    cp ${f}_libretro.* /out/cores
    popd
    popd
done
popd
fi
