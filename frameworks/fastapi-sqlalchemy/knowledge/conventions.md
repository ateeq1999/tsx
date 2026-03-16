# FastAPI + SQLAlchemy — Conventions

- Models in `app/models/`, schemas in `app/schemas/`, CRUD in `app/crud/`, routers in `app/routers/`
- Register routers with `app.include_router(router, prefix="/api")` in `app/main.py`
- Inject `AsyncSession` via `Depends(get_db)` — never import the session directly
- Use `model_config = ConfigDict(from_attributes=True)` on all response schemas
