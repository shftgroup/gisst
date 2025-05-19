#!/bin/sh

curl \
  -X POST 'localhost:7700/indexes' \
  -H "Authorization: Bearer $MEILI_API_KEY" \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "uid": "work",
    "primaryKey": "work_id"
  }'
curl \
  -X POST 'localhost:7700/indexes' \
  -H "Authorization: Bearer $MEILI_API_KEY" \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "uid": "instance",
    "primaryKey": "instance_id"
  }'
curl \
  -X POST 'localhost:7700/indexes' \
  -H "Authorization: Bearer $MEILI_API_KEY" \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "uid": "state",
    "primaryKey": "state_id"
  }'
curl \
  -X POST 'localhost:7700/indexes' \
  -H "Authorization: Bearer $MEILI_API_KEY" \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "uid": "save",
    "primaryKey": "save_id"
  }'
curl \
  -X POST 'localhost:7700/indexes' \
  -H "Authorization: Bearer $MEILI_API_KEY" \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "uid": "replay",
    "primaryKey": "replay_id"
  }'
curl \
  -X POST 'localhost:7700/indexes' \
  -H "Authorization: Bearer $MEILI_API_KEY" \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "uid": "creator",
    "primaryKey": "creator_id"
  }'
