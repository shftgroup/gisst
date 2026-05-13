#!/bin/bash

SCRIPT_VERSION="1.0.0"

set -o pipefail
set -e
set -v

MANIFEST_FILE="${MANIFEST_FILE:-/manifest.json}"
OUT_DIR="${OUT_DIR:-/out/}"
FILE_ROOT="${FILE_ROOT:-/files/}"

git --version
git config --global advice.detachedHead false

function die
{
    local message=$1
    [ -z "$message" ] && message="Died"
    echo "${BASH_SOURCE[1]}: line ${BASH_LINENO[0]}: ${FUNCNAME[1]}: $message." >&2
    exit 1
}

[ -e "$MANIFEST_FILE" ] || die "No manifest file available in container at $MANIFEST_FILE"

function getrepo
{
    local dir=$1
    local jq_prefix=$2
    local repo=$(jq -re "${jq_prefix}.repo" "$MANIFEST_FILE") || die "Could not get repository url"
    local vsn=$(jq -re "${jq_prefix}.version" "$MANIFEST_FILE") || die "Could not get repository version"
    if [ -e $dir ]; then
        pushd $dir
        if [ $vsn != $(git rev-parse HEAD) ]; then
            echo "Checkout of $dir exists with wrong version $(git rev-parse HEAD) (expected $vsn); removing..."
            popd
            rm -r $dir
        else
            echo "$dir already checked out at correct version"
            popd
            return 0;
        fi
    fi
    git clone --recursive $repo $dir
    pushd $dir
    git checkout $vsn
    popd
    return $?
}

# check script, rust, and emsdk versions from env var against manifest
MANIFEST_SCRIPT_VERSION=$(jq -r ".script_version" "$MANIFEST_FILE")
[ $MANIFEST_SCRIPT_VERSION = "${SCRIPT_VERSION}" ] || die "Container built with wrong script version ${SCRIPT_VERSION} for manifest $MANIFEST_SCRIPT_VERSION"
MANIFEST_RUST_VERSION=$(jq -r ".rust_version" "$MANIFEST_FILE")
[ $MANIFEST_RUST_VERSION = "${RUST_VERSION}" ] || die "Container built with wrong rust version ${RUST_VERSION} for manifest $MANIFEST_RUST_VERSION"
MANIFEST_EMSDK_VERSION=$(jq -r ".emsdk_version" "$MANIFEST_FILE")
[ $MANIFEST_EMSDK_VERSION = "${EMSDK_VERSION}" ] || die "Container built with wrong emsdk version ${EMSDK_VERSION} for manifest $MANIFEST_EMSDK_VERSION"

mkdir -p "$OUT_DIR/cores"

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
cp ../assets_minimal.zip "$OUT_DIR"/assets_minimal.zip
popd
fi

if [ ${FETCH_CORES:-1} -eq 1 ] ; then
mkdir -p ra-build
pushd ra-build

getrepo v86 '.v86'
getrepo ra '.retroarch'

CORENAMES=$(jq -re '.retroarch.cores | to_entries[] | select(.value.skip|not) | .key' "$MANIFEST_FILE")

for corename in $CORENAMES; do
    getrepo $corename ".retroarch.cores.${corename}"
done

popd
fi

if [ ${BUILD_CORES:-1} -eq 1 ]; then
pushd ra-build

pushd ra
rm -rf obj-emscripten
popd

build_args=($((jq -r '.retroarch.build_args | @sh' "$MANIFEST_FILE") | tr -d \'))

RETROARCH_VERSION=$(jq -r '.retroarch.version' "$MANIFEST_FILE")
for f in $CORENAMES v86; do
    if [ $f = "ra" ]; then
        continue
    fi
    pushd $f

    ASYNC=0
    if jq -re ".retroarch.cores.$f.async == true" "$MANIFEST_FILE"; then
        ASYNC=1
    fi
    EXTRA_ARGS=$(jq -re ".retroarch.cores.$f.extra_build_args // [] | join(\" \")" "$MANIFEST_FILE")

    if [ $f = "v86" ]
    then
        # make clean
        WASM_OPT=true PATH="${PATH}:${EMSDK}/upstream/bin" make all -j || die "could not build v86"
        cp build/libv86.js build/v86.wasm "$OUT_DIR"
        for biosfile in $(jq -r ".v86.dependencies // {} | keys[]" "$MANIFEST_FILE"); do
            bios_src=$(jq -r ".v86.dependencies[\"$biosfile\"]" "$MANIFEST_FILE")
            cp "${FILE_ROOT}/${bios_src}" "${OUT_DIR}/${biosfile}"
        done
        ENTRYPOINTS_STR='["libv86.js"]'
        DEPENDENCIES_STR=$(jq --compact-output '[.v86.dependencies // {} | keys[]]+["v86.wasm"]' "$MANIFEST_FILE")
        jq "del(.[\"retroarch\",\"emsdk_version\"]) + {\"core_name\":\"v86\", \"entrypoints\":${ENTRYPOINTS_STR}, \"dependencies\":${DEPENDENCIES_STR}}" "$MANIFEST_FILE" > "$OUT_DIR"/v86.json
        sha1sum "$OUT_DIR"/v86.json | cut -f 1 -d ' ' > "$OUT_DIR"/v86.hash
        echo v86
        cat "$OUT_DIR"/v86.json
        cat "$OUT_DIR"/v86.hash
        popd
        continue
    elif [ $f = "sameboy" ]
    then
        rgbds_repo=$(jq -r '.retroarch.cores.sameboy.rgbds_repo' "$MANIFEST_FILE")
        rgbds_version=$(jq -r '.retroarch.cores.sameboy.rgbds_version' "$MANIFEST_FILE")
        git clone $rgbds_repo rgbds && pushd rgbds && git checkout $rgbds_version && popd
        make -C rgbds -j || die "Could not build rgbds"
        PATH="./rgbds:${PATH}" make -j CONF=release bootroms || die "could not build sameboy bootroms"
        PATH="./rgbds:${PATH}" emmake make CONF=release platform=emscripten pthread=4 STATIC_LINKING=1 ASYNC=$ASYNC $EXTRA_ARGS -j libretro || die "could not build core ${f}"
        cp build/bin/${f}_libretro_emscripten.bc ../ra/libretro_emscripten.bc
    elif [ -f Makefile.libretro ]
    then
        # emmake make -f Makefile.libretro platform=emscripten clean
        emmake make -f Makefile.libretro platform=emscripten pthread=4 STATIC_LINKING=1 ASYNC=$ASYNC $EXTRA_ARGS -j || die "could not build core ${f}"
        cp ${f}_libretro_emscripten.bc ../ra/libretro_emscripten.bc
    elif [ -d libretro ] && [ -f libretro/Makefile ]
    then
        pushd libretro
        # emmake make platform=emscripten clean
        emmake make platform=emscripten pthread=4 STATIC_LINKING=1 ASYNC=$ASYNC $EXTRA_ARGS -j || die "could not build core ${f}"
        cp ${f}_libretro_emscripten.bc ../../ra/libretro_emscripten.bc
        popd
    else
        # emmake make platform=emscripten clean
        emmake make platform=emscripten pthread=4 STATIC_LINKING=1 ASYNC=$ASYNC $EXTRA_ARGS -j || die "could not build core ${f}"
        cp ${f}_libretro_emscripten.bc ../ra/libretro_emscripten.bc
    fi
    pushd ../ra
    cp libretro_emscripten.bc libretro_emscripten.a
    # flags may have changed so we should really clean the build here
    rm -rf obj-emscripten
    emmake make -f Makefile.emscripten LIBRETRO=$f clean
    emmake make -f Makefile.emscripten LIBRETRO=$f ASYNC=$ASYNC "${build_args[@]}" -j all || die "could not build RA dist for ${f}"
    cp ${f}_libretro.* "$OUT_DIR"/cores
    # compute hash from manifest (except for v86 and non-this-core RA cores)
    ENTRYPOINTS_STR="[\"${f}_libretro.js\"]"
    for depfile in $(jq -r ".retroarch.cores.${f}.dependencies // {} | keys[]" "$MANIFEST_FILE"); do
        dep_src=$(jq -r ".retroarch.cores.${f}.dependencies[\"$depfile\"]" "$MANIFEST_FILE")
        cp "${FILE_ROOT}/${dep_src}" "${OUT_DIR}/cores/${depfile}"
    done
    DEPENDENCIES_STR=$(jq --compact-output "[.retroarch.cores.${f}.dependencies // {} | keys[]] + [\"${f}_libretro.wasm\"]" "$MANIFEST_FILE")
    jq ".retroarch.cores = (.retroarch.cores | to_entries[] | select(.key == \"${f}\") | [.] | from_entries) | del(.[\"v86\",\"rust_version\"]) + {\"core_name\":\"${f}\", \"entrypoints\":${ENTRYPOINTS_STR}, \"dependencies\":${DEPENDENCIES_STR}}" "$MANIFEST_FILE" > "${OUT_DIR}/cores/${f}_libretro.json"
    sha1sum "$OUT_DIR"/cores/${f}_libretro.json | cut -f 1 -d ' ' > "$OUT_DIR"/cores/${f}_libretro.hash
    echo "${f}_libretro"
    cat "$OUT_DIR"/cores/${f}_libretro.json
    cat "$OUT_DIR"/cores/${f}_libretro.hash
    popd
    popd
done
popd
fi
