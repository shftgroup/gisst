#!/bin/bash

BRANCH=${1//\//-}
if [ "$BRANCH" = "" ] || [ "$BRANCH" = "eph" ]; then
    echo "no branch or invalid branch specified"
    exit 1
fi
PORT=$(cat "${BRANCH}/port")
docker compose down --rmi local -f "${BRANCH}/docker-compose.yml" -p "$BRANCH" -t 5
rm "eph/nginx/${BRANCH}.nginx.frag"
rm -rf "./${BRANCH}"

