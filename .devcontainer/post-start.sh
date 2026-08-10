#!/bin/bash

set -e

if [ -f .devcontainer/user/post-start.sh ]; then
    echo "Found user script"
    ./.devcontainer/user/post-start.sh
fi

sudo service nginx start
