#!/bin/sh

docker ps -a -q -f "name=^gisst_devcontainer" | xargs docker stop
docker ps -a -q -f "name=^gisst_devcontainer" | xargs docker rm
