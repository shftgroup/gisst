#!/bin/sh

podman ps -a -q -f "name=^gisst_devcontainer" | xargs podman stop
if [ ${DELETE:=0} = 1 ]; then
podman ps -a -q -f "name=^gisst_devcontainer" | xargs podman rm
fi
