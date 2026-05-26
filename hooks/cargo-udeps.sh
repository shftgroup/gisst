#!/bin/bash

cd backend
cargo +nightly udeps --workspace --all-targets --all-features
