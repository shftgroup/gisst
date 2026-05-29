#!/bin/bash

cd frontend

npm ci && npm run build --workspaces && npm run dist --if-present --workspaces && npm run check --workspaces --if-present
