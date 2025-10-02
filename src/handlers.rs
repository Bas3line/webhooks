use axum::{
    extract::{ConnectInfo, Path, State},
    http::{HeaderMap, StatusCode},
    response::Json,
    body::Bytes,
};
use chrono::Utc;
use serde_json::Value;
use std::{collections::HashMap, net::SocketAddr};
use tracing::{info, warn, error};
use uuid::Uuid;

use crate::{AppState, models::WebhookEvent};
use crate::middleware::verify_webhook_signature;

pub async fn receive_webhook(
    Path(endpoint): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    body: Bytes,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let webhook = match state.db.get_webhook_by_endpoint(&endpoint).await {
        Ok(Some(webhook)) => webhook,
        Ok(None) => {
            warn!("Webhook endpoint '{}' not found or inactive", endpoint);
            return Err(StatusCode::NOT_FOUND);
        }
        Err(e) => {
            error!("Database error: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    if let Err(status) = verify_webhook_signature(&headers, &body, Some(&webhook.secret)).await {
        warn!("Webhook signature verification failed for endpoint: {}", endpoint);
        return Err(status);
    }

    let body_json: Value = match serde_json::from_slice(&body) {
        Ok(json) => json,
        Err(e) => {
            warn!("Failed to parse webhook body as JSON: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    let headers_map: HashMap<String, String> = headers
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    let event_type = detect_webhook_type(&headers, &body_json);

    let event = WebhookEvent {
        id: Uuid::new_v4(),
        user_id: webhook.user_id,
        webhook_id: Some(webhook.id),
        timestamp: Utc::now(),
        headers: headers_map,
        body: body_json.clone(),
        source_ip: Some(addr.ip().to_string()),
        endpoint: endpoint.clone(),
        event_type: Some(event_type.clone()),
        status_code: 200,
        created_at: Utc::now(),
    };

    info!(
        "Received webhook on endpoint '{}' from {}: {}",
        endpoint,
        addr.ip(),
        event_type
    );

    if let Err(e) = state.db.store_event(&event).await {
        error!("Failed to store webhook event: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    process_webhook_by_type(&headers, &body_json, &endpoint).await;

    Ok(Json(serde_json::json!({
        "status": "received",
        "id": event.id,
        "timestamp": event.timestamp,
        "endpoint": endpoint,
        "webhook_id": webhook.id
    })))
}

pub async fn receive_generic_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    body: Bytes,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let body_json: Value = match serde_json::from_slice(&body) {
        Ok(json) => json,
        Err(e) => {
            warn!("Failed to parse webhook body as JSON: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    let headers_map: HashMap<String, String> = headers
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    let event_type = detect_webhook_type(&headers, &body_json);

    let event = WebhookEvent {
        id: Uuid::new_v4(),
        user_id: None,
        webhook_id: None,
        timestamp: Utc::now(),
        headers: headers_map,
        body: body_json.clone(),
        source_ip: Some(addr.ip().to_string()),
        endpoint: "generic".to_string(),
        event_type: Some(event_type.clone()),
        status_code: 200,
        created_at: Utc::now(),
    };

    info!(
        "Received generic webhook from {}: {}",
        addr.ip(),
        event_type
    );

    if let Err(e) = state.db.store_event(&event).await {
        error!("Failed to store webhook event: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    process_webhook_by_type(&headers, &body_json, "generic").await;

    Ok(Json(serde_json::json!({
        "status": "received",
        "id": event.id,
        "timestamp": event.timestamp,
        "endpoint": "generic"
    })))
}

fn detect_webhook_type(headers: &HeaderMap, body: &Value) -> String {
    if headers.contains_key("x-github-event") {
        let event_type = headers
            .get("x-github-event")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("unknown");
        format!("GitHub {}", event_type)
    } else if headers.contains_key("x-gitlab-event") {
        let event_type = headers
            .get("x-gitlab-event")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("unknown");
        format!("GitLab {}", event_type)
    } else if headers.contains_key("x-bitbucket-event") {
        let event_type = headers
            .get("x-bitbucket-event")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("unknown");
        format!("Bitbucket {}", event_type)
    } else if let Some(stripe_event) = body.get("type") {
        format!("Stripe {}", stripe_event.as_str().unwrap_or("unknown"))
    } else if body.get("zen").is_some() {
        "GitHub Ping".to_string()
    } else if body.get("hook_id").is_some() {
        "Generic Hook".to_string()
    } else {
        "Unknown".to_string()
    }
}

async fn process_webhook_by_type(headers: &HeaderMap, body: &Value, endpoint: &str) {
    match detect_webhook_type(headers, body).as_str() {
        event_type if event_type.starts_with("GitHub") => {
            process_github_webhook(headers, body, endpoint).await;
        }
        event_type if event_type.starts_with("GitLab") => {
            process_gitlab_webhook(headers, body, endpoint).await;
        }
        event_type if event_type.starts_with("Bitbucket") => {
            process_bitbucket_webhook(headers, body, endpoint).await;
        }
        event_type if event_type.starts_with("Stripe") => {
            process_stripe_webhook(headers, body, endpoint).await;
        }
        _ => {
            process_generic_webhook(headers, body, endpoint).await;
        }
    }
}

async fn process_github_webhook(headers: &HeaderMap, body: &Value, endpoint: &str) {
    let event_type = headers
        .get("x-github-event")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");

    match event_type {
        "push" => {
            if let Some(repo) = body.get("repository").and_then(|r| r.get("full_name")) {
                info!("GitHub push to {} on endpoint '{}'", repo, endpoint);
            }
        }
        "pull_request" => {
            if let Some(action) = body.get("action") {
                info!("GitHub PR {} on endpoint '{}'", action, endpoint);
            }
        }
        "ping" => {
            info!("GitHub ping received on endpoint '{}'", endpoint);
        }
        _ => {
            info!("GitHub {} event on endpoint '{}'", event_type, endpoint);
        }
    }
}

async fn process_gitlab_webhook(headers: &HeaderMap, body: &Value, endpoint: &str) {
    let event_type = headers
        .get("x-gitlab-event")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");

    match event_type {
        "Push Hook" => {
            if let Some(project) = body.get("project").and_then(|p| p.get("path_with_namespace")) {
                info!("GitLab push to {} on endpoint '{}'", project, endpoint);
            }
        }
        "Merge Request Hook" => {
            if let Some(action) = body.get("object_attributes").and_then(|o| o.get("action")) {
                info!("GitLab MR {} on endpoint '{}'", action, endpoint);
            }
        }
        _ => {
            info!("GitLab {} event on endpoint '{}'", event_type, endpoint);
        }
    }
}

async fn process_bitbucket_webhook(headers: &HeaderMap, _body: &Value, endpoint: &str) {
    let event_type = headers
        .get("x-event-key")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");

    info!("Bitbucket {} event on endpoint '{}'", event_type, endpoint);
}

async fn process_stripe_webhook(_headers: &HeaderMap, body: &Value, endpoint: &str) {
    if let Some(event_type) = body.get("type") {
        info!("Stripe {} event on endpoint '{}'", event_type, endpoint);
    }
}

async fn process_generic_webhook(_headers: &HeaderMap, _body: &Value, endpoint: &str) {
    info!("Generic webhook received on endpoint '{}'", endpoint);
}