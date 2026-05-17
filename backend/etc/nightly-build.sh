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
