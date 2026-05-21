#!/bin/bash

cd backend
cargo sqlx prepare --all --check --workspace -- --all-targets
