#!/bin/bash

rsync -avh simple/* retroarch-web/
cd retroarch-web || exit
echo "http://localhost:8080/embed_one.html"
chromium --new-window "http://localhost:8080/embed_one.html" &
python -m http.server 8080
