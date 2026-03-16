#!/usr/bin/env bash
# Reproduces the with-shadcn example from scratch.
set -e

tsx init --name with-shadcn --stack tanstack-start,drizzle-pg,better-auth,shadcn

tsx run add:auth-setup --json '{ "providers": ["github"], "two_factor": false }'

tsx run add:schema --json '{
  "name": "posts",
  "fields": [
    { "name": "title",   "type": "string", "required": true },
    { "name": "content", "type": "string" },
    { "name": "userId",  "type": "number", "required": true }
  ],
  "timestamps": true
}'

tsx run add:server-fn --json '{
  "name": "posts",
  "operations": ["list", "create", "delete"],
  "table": "postsTable",
  "auth": true
}'

tsx run add:query --json '{ "name": "posts", "operations": ["list", "create", "delete"] }'

# shadcn generators
tsx run add:ui-form --json '{
  "name": "posts",
  "fields": [
    { "name": "title",   "type": "string" },
    { "name": "content", "type": "string" }
  ]
}'

tsx run add:ui-data-table --json '{
  "name": "posts",
  "fields": [
    { "name": "title",     "type": "string" },
    { "name": "createdAt", "type": "date" }
  ]
}'
