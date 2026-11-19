#!/bin/bash
set -e
set -v
# usage: eph.sh git-branch-name
# to stop it later: eph-down.sh git-branch-name

declare -i PORT

inc_port() {
  PORT=$(cat port || echo 3001)
  if (( $PORT > 3999 )); then
    PORT=3001
  fi
  echo $(( $PORT + 1 )) > port
  echo $PORT
}

export PORT=$(inc_port)
echo "PORT is $PORT"

export GIT_BRANCH=$1
export BRANCH=${GIT_BRANCH//\//-}
export ROOT=gisst-test
export WORKER_KEY="worker-$(pwgen -s1 16)"
if [ -z "${BRANCH}" ] || [ "$BRANCH" = "eph" ]; then
  echo "no branch or invalid branch name eph specified"
  exit 1
fi
if [ -d "${BRANCH}" ]; then
  cd "${BRANCH}"
  docker compose down --rmi local -t 5
  cd ..
  rm -rf "${BRANCH}"
fi
git clone --depth 1 --revision "${GIT_BRANCH}" ssh://forgejo@cafe.cs.pomona.edu:47147/shftgroup/gisst.git "$BRANCH"
cd "$BRANCH"
docker build -f Dockerfile.ci -t "gisst-eph:${BRANCH}" .
echo $PORT > port
echo $WORKER_KEY > worker
. ../secrets
envsubst '$PORT $BRANCH $ROOT' < ../eph/eph.nginx.t.frag > "/etc/nginx/eph/${BRANCH}.nginx.frag"

# passwordless sudo is required for this
sudo /usr/bin/nginx -s reload

cp ../eph/eph.nginx.t.conf "eph.nginx.conf"
envsubst < ../eph/eph.local.t.toml > "backend/config/local.toml"
envsubst < ../eph/eph.t.env > ".env"
cp .env backend/.env
envsubst '$PORT $BRANCH' < ../eph/eph.compose.t.yml > docker-compose.yml

COMPOSE_MENU=false docker compose up --wait
