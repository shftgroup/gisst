#!/bin/bash
set -e
cd backend
cargo audit
cargo deny check
cargo vet
osv-scanner scan source -r .
