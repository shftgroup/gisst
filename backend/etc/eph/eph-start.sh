#!/bin/bash

set -e
set -v
sudo nginx -t 
sudo service nginx start
cd /workspace/

mv /tmp/frontend/node_modules frontend/
cd frontend
npm ci
npm run build --workspaces --if-present
npm run dist --workspaces --if-present
cd ..
mv /tmp/backend/target backend/
cd backend
. .env
./reset.sh && cargo run --bin gisst-server --features dummy_auth 2>&1
