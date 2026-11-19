#!/bin/bash

BRANCH=${1//\//-}
if [ "$BRANCH" = "" ] || [ "$BRANCH" = "eph" ]; then
    echo "no branch or invalid branch specified"
    exit 1
fi
PORT=$(cat "${BRANCH}/port")
cd "${BRANCH}"
docker compose down --rmi local -t 5
cd ..
rm "/etc/nginx/eph/${BRANCH}.nginx.frag"
rm -rf "./${BRANCH}"

