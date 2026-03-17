use anyhow::Result;
use sqlx::PgPool;

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
    sqlx::query!(
        r#"INSERT INTO audit_log (action, package_name, version, user_id, author_name, ip_address, detail)
           VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
        action, package_name, version, user_id, author_name, ip,
        detail as Option<serde_json::Value>
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_audit_log(pool: &PgPool, limit: i64) -> Result<Vec<AuditEntry>> {
    let rows = sqlx::query!(
        r#"SELECT id, action, package_name, version, author_name, ip_address,
                  created_at::TEXT AS "created_at!"
           FROM audit_log
           ORDER BY created_at DESC
           LIMIT $1"#,
        limit
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| AuditEntry {
        id: r.id,
        action: r.action,
        package_name: r.package_name,
        version: r.version,
        author_name: r.author_name,
        ip_address: r.ip_address,
        created_at: r.created_at,
    }).collect())
}
