#!/bin/bash

cargo sqlx database reset -y --force
cargo sqlx prepare --all --check --workspace -- --all-targets
rm -rf storage/*
STOP_ON_ERR=true ./load_examples.sh
