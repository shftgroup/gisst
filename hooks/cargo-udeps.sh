#!/bin/bash

cd backend
cargo udeps --workspace --all-targets --all-features
