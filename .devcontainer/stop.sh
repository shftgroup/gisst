#!/bin/sh

docker ps -a -q -f "name=^gisst_devcontainer" | xargs docker stop
if [ ${DELETE:=0} = 1 ]; then
docker ps -a -q -f "name=^gisst_devcontainer" | xargs docker rm
fi
