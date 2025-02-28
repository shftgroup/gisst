@echo off

echo Rust backend server starting...
cd backend
start cmd /k cargo run --bin gisst-server --features dummy_auth

color 2

echo Building frontend workspaces

color 7
cd ../frontend
npm run --workspaces build
echo finished building frontend

echo running frontend embed
cd frontend-embed
npm run dev
