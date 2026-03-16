#!/usr/bin/env bash
# Reproduces the full-saas example from scratch.
set -e

tsx init --name full-saas --stack tanstack-start,drizzle-pg,better-auth,shadcn

# Auth
tsx run add:auth-setup --json '{ "providers": ["github", "google"], "two_factor": false }'
tsx run add:auth-guard --json '{ "name": "dashboard", "redirect_to": "/login" }'

# Schemas
tsx run add:schema --json '{
  "name": "organizations",
  "fields": [
    { "name": "name",   "type": "string", "required": true },
    { "name": "slug",   "type": "string", "required": true, "unique": true },
    { "name": "planId", "type": "string" }
  ],
  "timestamps": true
}'

tsx run add:schema --json '{
  "name": "memberships",
  "fields": [
    { "name": "userId",         "type": "number", "required": true },
    { "name": "organizationId", "type": "number", "required": true },
    { "name": "role",           "type": "string", "required": true }
  ],
  "timestamps": true
}'

tsx run add:schema --json '{
  "name": "users",
  "fields": [
    { "name": "name",  "type": "string", "required": true },
    { "name": "email", "type": "string", "required": true, "unique": true },
    { "name": "role",  "type": "string" }
  ],
  "timestamps": true
}'

# Server functions
tsx run add:server-fn --json '{ "name": "organizations", "operations": ["list","get","create","update","delete"], "table": "organizationsTable", "auth": true }'
tsx run add:server-fn --json '{ "name": "memberships",   "operations": ["list","create","delete"],                "table": "membershipsTable",   "auth": true }'
tsx run add:server-fn --json '{ "name": "users",         "operations": ["list","get","update"],                   "table": "usersTable",         "auth": true }'

# Queries
tsx run add:query --json '{ "name": "organizations", "operations": ["list","get","create","update","delete"] }'
tsx run add:query --json '{ "name": "memberships",   "operations": ["list","create","delete"] }'
tsx run add:query --json '{ "name": "users",         "operations": ["list","get","update"] }'

# UI
tsx run add:ui-form --json '{ "name": "organizations", "fields": [{ "name": "name", "type": "string" }, { "name": "slug", "type": "string" }] }'
tsx run add:ui-form --json '{ "name": "users",         "fields": [{ "name": "name", "type": "string" }, { "name": "role", "type": "string" }] }'
tsx run add:ui-data-table --json '{ "name": "organizations", "fields": [{ "name": "name", "type": "string" }, { "name": "slug", "type": "string" }] }'
tsx run add:ui-data-table --json '{ "name": "users",         "fields": [{ "name": "name", "type": "string" }, { "name": "email", "type": "string" }] }'
