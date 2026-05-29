#!/bin/bash

set -e
set -o pipefail

if [ ${TESTING:-0} -eq 0 ]; then
WORKDIR=${WORKDIR:-/tmp/gisst}
if [ -e $WORKDIR ]; then
    pushd $WORKDIR
    git pull origin main
else
    git clone --depth 1 ssh://forgejo@cafe.cs.pomona.edu:47147/shftgroup/gisst $WORKDIR
    pushd $WORKDIR
fi
cat $JOB_TOKEN | docker login cafe.cs.pomona.edu -u jcoa2018 --password-stdin
fi
echo "build ci"
docker build --platform linux/arm64,linux/amd64 \
	-f Dockerfile.ci \
        -t cafe.cs.pomona.edu/shftgroup/gisst-ci:latest\
	--push .
echo "build dev"
docker build --platform linux/arm64,linux/amd64 \
        -f Dockerfile.dev \
	-t cafe.cs.pomona.edu/shftgroup/gisst-dev:latest\
	--push .
echo "build dist"
docker build --platform linux/arm64,linux/amd64 \
	-f Dockerfile.dist \
	-t cafe.cs.pomona.edu/shftgroup/gisst-dist:latest\
	--push .
echo "build cores"
OLD_VSN=$(cat last_build || echo "")
NEW_VSN=$(cat manifest.json Dockerfile.dist dist-update-docker.sh | sha256sum | cut -d' ' -f1)
if [ "$OLD_VSN" != "$NEW_VSN" ]; then
  rsync -a ~/Projects/gisst/{psxbios,v86bios} .
  if [ -f build/assets_minimal.tgz ]; then
    ASSETS=0
  else
    ASSETS=1
  fi
  docker run --rm -v $(pwd)/build:/out -v $(pwd)/manifest.json:/manifest.json -v $(pwd)/v86bios:/files/v86bios -v $(pwd)/psxbios:/files/psxbios -v gisst-build:/gisst-build -e GET_ASSETS=$ASSETS cafe.cs.pomona.edu/shftgroup/gisst-dist:latest
  tar czvf cores.tgz build
  curl -u "jcoa2018:$JOB_TOKEN" --anyauth --request DELETE "https://cafe.cs.pomona.edu/api/packages/shftgroup/generic/cores/latest/cores.tgz" || echo "no existing package to delete"
  curl -u "jcoa2018:$JOB_TOKEN" --anyauth --upload-file cores.tgz "https://cafe.cs.pomona.edu/api/packages/shftgroup/generic/cores/latest/cores.tgz"
  echo $NEW_VSN > last_build
fi
