# Gin + GORM — Overview

Gin is a high-performance Go HTTP web framework. GORM is the Go ORM library.
This package generates GORM models, Gin controllers with CRUD handlers, and middleware.

## Key commands

| Command | What it generates |
|---|---|
| `add:model` | `internal/models/{{name}}.go` — GORM model struct |
| `add:controller` | `internal/controllers/{{name}}.go` — Gin CRUD controller |
| `add:middleware` | `internal/middleware/{{name}}.go` — Gin middleware function |

## Auto-migration

Register your models with `db.AutoMigrate(&models.YourModel{})` in your startup code.
