#!/bin/bash

cd backend
cargo sqlx prepare --check --workspace
