use anyhow::Result;
use sqlx::{PgPool, Row};

use crate::models::AuditEntry;

pub async fn insert_audit(
    pool: &PgPool,
    action: &str,
    package_name: &str,
    version: Option<&str>,
    user_id: Option<&str>,
    author_name: Option<&str>,
    ip: Option<&str>,
    detail: Option<serde_json::Value>,
) -> Result<()> {
    sqlx::query(
        r#"INSERT INTO audit_log (action, package_name, version, user_id, author_name, ip_address, detail)
           VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
    )
    .bind(action)
    .bind(package_name)
    .bind(version)
    .bind(user_id)
    .bind(author_name)
    .bind(ip)
    .bind(detail)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_audit_log(pool: &PgPool, limit: i64) -> Result<Vec<AuditEntry>> {
    let rows = sqlx::query(
        r#"SELECT id, action, package_name, version, author_name, ip_address,
                  created_at::TEXT AS created_at
           FROM audit_log
           ORDER BY created_at DESC
           LIMIT $1"#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| AuditEntry {
        id: r.get("id"),
        action: r.get("action"),
        package_name: r.get("package_name"),
        version: r.get("version"),
        author_name: r.get("author_name"),
        ip_address: r.get("ip_address"),
        created_at: r.get("created_at"),
    }).collect())
}
