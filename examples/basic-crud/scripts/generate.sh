#!/usr/bin/env bash
# Reproduces the basic-crud example from scratch.
# Requires: tsx CLI installed (cargo install tsx)
set -e

tsx init --name basic-crud --stack tanstack-start,drizzle-pg

tsx run add:schema --json '{
  "name": "products",
  "fields": [
    { "name": "title",       "type": "string",  "required": true },
    { "name": "description", "type": "string" },
    { "name": "price",       "type": "number",  "required": true },
    { "name": "inStock",     "type": "boolean" }
  ],
  "timestamps": true
}'

tsx run add:server-fn --json '{
  "name": "products",
  "operations": ["list", "get", "create", "update", "delete"],
  "table": "productsTable",
  "auth": false
}'

tsx run add:query --json '{
  "name": "products",
  "operations": ["list", "get", "create", "update", "delete"]
}'

tsx run add:form --json '{
  "name": "products",
  "fields": [
    { "name": "title",       "type": "string" },
    { "name": "description", "type": "string" },
    { "name": "price",       "type": "number" }
  ]
}'

tsx run add:table --json '{
  "name": "products",
  "fields": [
    { "name": "title", "type": "string" },
    { "name": "price", "type": "number" }
  ]
}'

tsx run add:page --json '{ "name": "products", "loader": true }'
