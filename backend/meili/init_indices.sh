#!/bin/sh

curl \
  -X POST "$MEILI_URL/indexes" \
  -H "Authorization: Bearer $MEILI_API_KEY" \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "uid": "instance",
    "primaryKey": "instance_id"
  }'
curl \
  -X PUT "$MEILI_URL/indexes/instance/settings/filterable-attributes" \
  -H "Authorization: Bearer $MEILI_API_KEY" \
  -H 'Content-Type: application/json' \
  --data-binary '[
    "work_platform"
  ]'
curl \
  -X POST "$MEILI_URL/indexes" \
  -H "Authorization: Bearer $MEILI_API_KEY" \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "uid": "state",
    "primaryKey": "state_id"
  }'
curl \
  -X PUT "$MEILI_URL/indexes/state/settings/filterable-attributes" \
  -H "Authorization: Bearer $MEILI_API_KEY" \
  -H 'Content-Type: application/json' \
  --data-binary '[
    "work_platform",
    "creator_id",
    "instance_id"
  ]'
curl \
  -X POST "$MEILI_URL/indexes" \
  -H "Authorization: Bearer $MEILI_API_KEY" \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "uid": "save",
    "primaryKey": "save_id"
  }'
curl \
  -X PUT "$MEILI_URL/indexes/save/settings/filterable-attributes" \
  -H "Authorization: Bearer $MEILI_API_KEY" \
  -H 'Content-Type: application/json' \
  --data-binary '[
    "work_platform",
    "creator_id",
    "instance_id"
  ]'
curl \
  -X POST "$MEILI_URL/indexes" \
  -H "Authorization: Bearer $MEILI_API_KEY" \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "uid": "replay",
    "primaryKey": "replay_id"
  }'
curl \
  -X PUT "$MEILI_URL/indexes/replay/settings/filterable-attributes" \
  -H "Authorization: Bearer $MEILI_API_KEY" \
  -H 'Content-Type: application/json' \
  --data-binary '[
    "work_platform",
    "creator_id",
    "instance_id"
  ]'
curl \
  -X PUT "$MEILI_URL/indexes" \
  -H "Authorization: Bearer $MEILI_API_KEY" \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "uid": "creator",
    "primaryKey": "creator_id"
  }'
