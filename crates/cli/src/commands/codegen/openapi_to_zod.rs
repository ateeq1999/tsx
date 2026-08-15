use crate::json::response::ResponseEnvelope;

pub fn codegen_openapi_to_zod(spec: String, out: Option<String>, _verbose: bool) -> ResponseEnvelope {
    let out_path = out.unwrap_or_else(|| "src/lib/api-schemas.ts".to_string());
    ResponseEnvelope::success(
        "codegen openapi-to-zod",
        serde_json::json!({
            "spec": spec,
            "output": out_path,
            "status": "To generate Zod schemas from your OpenAPI spec, run: npx openapi-zod-client@latest <spec> -o <output>",
            "recommended_tool": "openapi-zod-client",
            "install": "npm install -D openapi-zod-client",
        }),
        0,
    )
    .with_next_steps(vec![
        format!("npx openapi-zod-client {} -o {}", spec, out_path),
        "Add the generated file to your version control".to_string(),
    ])
}
