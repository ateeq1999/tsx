use hmac::{Hmac, Mac};
use sha2::Sha256;
use sqlx::PgPool;
use tracing::warn;

use crate::db;

/// Fire all active webhooks subscribed to `event` asynchronously.
/// Spawns a task per webhook; failures are logged but do not affect the caller.
pub fn fire(pool: PgPool, event: &'static str, payload: serde_json::Value) {
    tokio::spawn(async move {
        let webhooks = match db::get_active_webhooks_for_event(&pool, event).await {
            Ok(whs) => whs,
            Err(e) => {
                warn!(error = %e, "Failed to fetch webhooks for event {}", event);
                return;
            }
        };

        let client = match reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .user_agent("tsx-registry/1.0")
            .build()
        {
            Ok(c) => c,
            Err(e) => {
                warn!(error = %e, "Failed to build HTTP client for webhook delivery");
                return;
            }
        };

        let body = payload.to_string();

        for wh in webhooks {
            let mut req = client
                .post(&wh.url)
                .header("Content-Type", "application/json")
                .header("X-TSX-Event", event);

            if let Some(ref secret) = wh.secret {
                let sig = hmac_sha256(secret.as_bytes(), body.as_bytes());
                req = req.header("X-TSX-Signature-256", format!("sha256={sig}"));
            }

            match req.body(body.clone()).send().await {
                Ok(resp) if resp.status().is_success() => {
                    tracing::debug!(url = %wh.url, event, "Webhook delivered");
                }
                Ok(resp) => {
                    warn!(url = %wh.url, status = %resp.status(), "Webhook delivery non-2xx");
                }
                Err(e) => {
                    warn!(url = %wh.url, error = %e, "Webhook delivery failed");
                }
            }
        }
    });
}

fn hmac_sha256(key: &[u8], data: &[u8]) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC accepts any key length");
    mac.update(data);
    hex::encode(mac.finalize().into_bytes())
}
