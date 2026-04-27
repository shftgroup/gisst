#!/bin/bash

cargo sqlx database reset -y --force
cargo sqlx prepare --workspace
rm -rf storage/*
cargo run --bin gisst-cli -- init-indices
STOP_ON_ERR=true ./load_examples.sh
