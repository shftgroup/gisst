#!/bin/bash

set -e

[ -f .devcontainer/certs/local.crt ] || (mkdir -p .devcontainer/certs && openssl req -x509 -nodes -days 365 -newkey rsa:2048 -keyout .devcontainer/certs/local.key -out .devcontainer/certs/local.crt -subj '/CN=localhost')
