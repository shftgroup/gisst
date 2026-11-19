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
cd backend
if ($DCX rustup component list --installed | grep rust-std); then
    $DCX cargo sqlx prepare --workspace --all -- --all-targets
    $DCX cargo clippy --all-targets --workspace -- -Dclippy::pedantic -Dwarnings
    $DCX cargo clippy --all-targets --workspace --all-features -- -Dclippy::pedantic -Dwarnings
    $DCX cargo test --all-targets --workspace
    $DCX cargo test --all-targets --workspace --all-features
fi
