# FastAPI + SQLAlchemy — Overview

FastAPI is a modern Python web framework. SQLAlchemy 2.0 provides async ORM support.
This package generates models, Pydantic schemas, CRUD services, and APIRouter files.

## Key commands

| Command | What it generates |
|---|---|
| `add:model` | `app/models/{{name}}.py` — SQLAlchemy mapped class |
| `add:schema` | `app/schemas/{{name}}.py` — Pydantic v2 Create/Update/Response schemas |
| `add:crud` | `app/crud/{{name}}.py` — async CRUD service |
| `add:router` | `app/routers/{{name}}.py` — APIRouter with list/get/create/update/delete |

## Async by default

All generated code uses `AsyncSession` and `asyncpg`. Pass `async_mode: false` to get sync sessions.
