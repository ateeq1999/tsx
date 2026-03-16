# Gin + GORM — Conventions

- Models in `internal/models/`, controllers in `internal/controllers/`, middleware in `internal/middleware/`
- Embed `gorm.Model` to get ID, CreatedAt, UpdatedAt, and soft-delete DeletedAt for free
- Inject `*gorm.DB` via constructor — never use a global db variable
- Use `c.ShouldBindJSON(&payload)` for request binding, return `gin.H{"error": msg}` on errors
