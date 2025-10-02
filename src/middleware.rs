use axum::http::{HeaderMap, StatusCode};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use tracing::{debug, warn};

type HmacSha256 = Hmac<Sha256>;

pub async fn verify_webhook_signature(
    headers: &HeaderMap,
    body: &[u8],
    secret: Option<&String>,
) -> Result<(), StatusCode> {
    let Some(secret) = secret else {
        debug!("No secret configured, skipping signature verification");
        return Ok(());
    };

    let signature = headers
        .get("x-hub-signature-256")
        .or_else(|| headers.get("x-signature-256"))
        .or_else(|| headers.get("x-gitlab-token"))
        .and_then(|h| h.to_str().ok());

    let Some(signature) = signature else {
        warn!("No signature header found");
        return Err(StatusCode::UNAUTHORIZED);
    };

    let signature = signature.strip_prefix("sha256=").unwrap_or(signature);

    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    mac.update(body);
    let expected = hex::encode(mac.finalize().into_bytes());

    if signature.eq_ignore_ascii_case(&expected) {
        Ok(())
    } else {
        warn!("Signature verification failed");
        Err(StatusCode::UNAUTHORIZED)
    }
}