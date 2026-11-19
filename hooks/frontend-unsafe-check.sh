#!/bin/bash
set -e

DCX=$(which dcx || echo "")
if [ -x $DCX ]; then
    echo "using dcx to run in container"
elif [ $USER == "ubuntu" ]; then
    echo "running in container"
else
    echo "skipping unsafe checks, can't run commands in container"
    exit 0
fi
cd frontend
$DCX npm run test --workspaces --if-present
