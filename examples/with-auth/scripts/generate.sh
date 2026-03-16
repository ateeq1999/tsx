#!/usr/bin/env bash
# Reproduces the with-auth example from scratch.
set -e

tsx init --name with-auth --stack tanstack-start,drizzle-pg,better-auth

tsx run add:auth-setup --json '{
  "providers": ["github"],
  "two_factor": false
}'

tsx run add:auth-guard --json '{
  "name": "dashboard",
  "redirect_to": "/login"
}'

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

tsx run add:query --json '{
  "name": "posts",
  "operations": ["list", "create", "delete"]
}'

tsx run add:table --json '{
  "name": "posts",
  "fields": [
    { "name": "title",  "type": "string" },
    { "name": "userId", "type": "number" }
  ]
}'
