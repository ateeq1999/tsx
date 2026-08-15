use crate::json::response::ResponseEnvelope;

pub fn codegen_drizzle_to_zod(_verbose: bool) -> ResponseEnvelope {
    ResponseEnvelope::success(
        "codegen drizzle-to-zod",
        serde_json::json!({
            "status": "drizzle-zod integration",
            "install": "npm install drizzle-zod",
            "usage": "import { createInsertSchema, createSelectSchema } from 'drizzle-zod'",
            "example": "export const insertUserSchema = createInsertSchema(usersTable)\nexport const selectUserSchema = createSelectSchema(usersTable)",
        }),
        0,
    )
    .with_next_steps(vec![
        "npm install drizzle-zod".to_string(),
        "Import createInsertSchema / createSelectSchema from 'drizzle-zod' in your schema files".to_string(),
    ])
}
