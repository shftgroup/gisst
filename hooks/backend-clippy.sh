#!/bin/bash

cd backend

cargo clippy --all-targets --all-features -- -Dclippy::pedantic
