use axum::{
    extract::{Path, Query, State},
    http::{StatusCode, HeaderMap},
    response::Json,
};
use serde::Deserialize;
use sqlx::Row;
use uuid::Uuid;
use crate::{AppState, models::*};

#[derive(Deserialize)]
pub struct ListQuery {
    #[allow(dead_code)]
    pub limit: Option<i64>,
    #[allow(dead_code)]
    pub offset: Option<i64>,
    #[allow(dead_code)]
    pub webhook: Option<Uuid>,
}

async fn get_user_from_session(headers: &HeaderMap, state: &AppState) -> Option<Uuid> {
    let cookie_header = headers.get("cookie")?.to_str().ok()?;

    for cookie in cookie_header.split(';') {
        let cookie = cookie.trim();
        if cookie.starts_with("session_id=") {
            let session_id = cookie.strip_prefix("session_id=")?;
            if let Ok(Some(user_id)) = state.db.get_session_user(session_id).await {
                return Some(user_id);
            }
        }
    }
    None
}

pub async fn create_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateWebhookRequest>,
) -> Result<Json<WebhookResponse>, StatusCode> {
    let user_id = match get_user_from_session(&headers, &state).await {
        Some(user_id) => user_id,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    let secret = generate_secret();

    match state.db.create_webhook(
        Some(user_id),
        &request.name,
        &request.endpoint,
        &secret,
        request.description.as_deref(),
    ).await {
        Ok(webhook) => {
            let response = WebhookResponse {
                id: webhook.id,
                name: webhook.name,
                endpoint: webhook.endpoint.clone(),
                url: format!("{}/webhook/{}", state.config.base_url, webhook.endpoint),
                secret: webhook.secret,
                description: webhook.description,
                is_active: webhook.is_active,
                created_at: webhook.created_at,
                updated_at: webhook.updated_at,
                event_count: 0,
            };
            Ok(Json(response))
        },
        Err(err) => {
            eprintln!("Database error creating webhook: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn list_webhooks(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<WebhookResponse>>, StatusCode> {
    let user_id = match get_user_from_session(&headers, &state).await {
        Some(user_id) => user_id,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    match state.db.get_webhooks_by_user(user_id).await {
        Ok(webhooks) => {
            let responses: Vec<WebhookResponse> = webhooks.into_iter().map(|webhook| {
                WebhookResponse {
                    id: webhook.id,
                    name: webhook.name,
                    endpoint: webhook.endpoint.clone(),
                    url: format!("{}/webhook/{}", state.config.base_url, webhook.endpoint),
                    secret: webhook.secret,
                    description: webhook.description,
                    is_active: webhook.is_active,
                    created_at: webhook.created_at,
                    updated_at: webhook.updated_at,
                    event_count: 0,
                }
            }).collect();
            Ok(Json(responses))
        },
        Err(err) => {
            eprintln!("Database error listing webhooks: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn list_events(
    Query(params): Query<ListQuery>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<WebhookEvent>>, StatusCode> {
    // For now, allow unauthenticated access to view all events
    // In production, you might want to restrict this
    match state.db.get_all_events(params.limit, params.offset).await {
        Ok(events) => Ok(Json(events)),
        Err(err) => {
            eprintln!("Database error listing events: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_webhook(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<WebhookResponse>, StatusCode> {
    let user_id = match get_user_from_session(&headers, &state).await {
        Some(user_id) => user_id,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    match state.db.get_webhook(id, user_id).await {
        Ok(Some(webhook)) => {
            let event_count = state.db.get_webhook_event_count(webhook.id, user_id).await.unwrap_or(0);
            let response = WebhookResponse {
                id: webhook.id,
                name: webhook.name,
                endpoint: webhook.endpoint.clone(),
                url: format!("{}/webhook/{}", state.config.base_url, webhook.endpoint),
                secret: webhook.secret,
                description: webhook.description,
                is_active: webhook.is_active,
                created_at: webhook.created_at,
                updated_at: webhook.updated_at,
                event_count,
            };
            Ok(Json(response))
        },
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(err) => {
            eprintln!("Database error getting webhook: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn update_webhook(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<UpdateWebhookRequest>,
) -> Result<Json<WebhookResponse>, StatusCode> {
    let user_id = match get_user_from_session(&headers, &state).await {
        Some(user_id) => user_id,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    match state.db.update_webhook(
        id,
        user_id,
        request.name.as_deref(),
        request.description.as_deref(),
        request.is_active,
    ).await {
        Ok(Some(webhook)) => {
            let event_count = state.db.get_webhook_event_count(webhook.id, user_id).await.unwrap_or(0);
            let response = WebhookResponse {
                id: webhook.id,
                name: webhook.name,
                endpoint: webhook.endpoint.clone(),
                url: format!("{}/webhook/{}", state.config.base_url, webhook.endpoint),
                secret: webhook.secret,
                description: webhook.description,
                is_active: webhook.is_active,
                created_at: webhook.created_at,
                updated_at: webhook.updated_at,
                event_count,
            };
            Ok(Json(response))
        },
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(err) => {
            eprintln!("Database error updating webhook: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn delete_webhook(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<StatusCode, StatusCode> {
    let user_id = match get_user_from_session(&headers, &state).await {
        Some(user_id) => user_id,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    match state.db.delete_webhook(id, user_id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(err) => {
            eprintln!("Database error deleting webhook: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn regenerate_secret(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<WebhookResponse>, StatusCode> {
    let user_id = match get_user_from_session(&headers, &state).await {
        Some(user_id) => user_id,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    let new_secret = generate_secret();

    match state.db.update_webhook_secret(id, user_id, &new_secret).await {
        Ok(Some(webhook)) => {
            let event_count = state.db.get_webhook_event_count(webhook.id, user_id).await.unwrap_or(0);
            let response = WebhookResponse {
                id: webhook.id,
                name: webhook.name,
                endpoint: webhook.endpoint.clone(),
                url: format!("{}/webhook/{}", state.config.base_url, webhook.endpoint),
                secret: webhook.secret,
                description: webhook.description,
                is_active: webhook.is_active,
                created_at: webhook.created_at,
                updated_at: webhook.updated_at,
                event_count,
            };
            Ok(Json(response))
        },
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(err) => {
            eprintln!("Database error regenerating secret: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_event(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<WebhookEvent>, StatusCode> {
    let user_id = match get_user_from_session(&headers, &state).await {
        Some(user_id) => user_id,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    match state.db.get_event(id, user_id).await {
        Ok(Some(event)) => Ok(Json(event)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(err) => {
            eprintln!("Database error getting event: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

fn generate_secret() -> String {
    use rand::Rng;
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();

    (0..32)
        .map(|_| {
            let idx = rng.gen_range(0..CHARS.len());
            CHARS[idx] as char
        })
        .collect()
}