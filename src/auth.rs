use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use bcrypt::{hash, verify, DEFAULT_COST};
use uuid::Uuid;

use crate::{models::*, AppState};


pub async fn register(
    State(state): State<AppState>,
    Json(request): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    if request.username.len() < 3 || request.username.len() > 50 {
        return Ok(Json(AuthResponse {
            success: false,
            message: "Username must be between 3 and 50 characters".to_string(),
            user_id: None,
        }));
    }

    if request.password.len() < 6 {
        return Ok(Json(AuthResponse {
            success: false,
            message: "Password must be at least 6 characters".to_string(),
            user_id: None,
        }));
    }

    let existing_user = state.db.get_user_by_username(&request.username).await;
    if existing_user.is_ok() && existing_user.unwrap().is_some() {
        return Ok(Json(AuthResponse {
            success: false,
            message: "Username already exists".to_string(),
            user_id: None,
        }));
    }

    let password_hash = match hash(&request.password, DEFAULT_COST) {
        Ok(hash) => hash,
        Err(_) => {
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    match state.db.create_user(&request.username, &password_hash).await {
        Ok(user) => Ok(Json(AuthResponse {
            success: true,
            message: "User registered successfully".to_string(),
            user_id: Some(user.id),
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<(axum::http::HeaderMap, Json<AuthResponse>), StatusCode> {
    let user = match state.db.get_user_by_username(&request.username).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Ok((axum::http::HeaderMap::new(), Json(AuthResponse {
                success: false,
                message: "Invalid username or password".to_string(),
                user_id: None,
            })));
        }
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    if !user.is_active {
        return Ok((axum::http::HeaderMap::new(), Json(AuthResponse {
            success: false,
            message: "Account is disabled".to_string(),
            user_id: None,
        })));
    }

    let is_valid = match verify(&request.password, &user.password_hash) {
        Ok(valid) => valid,
        Err(_) => false,
    };

    if !is_valid {
        return Ok((axum::http::HeaderMap::new(), Json(AuthResponse {
            success: false,
            message: "Invalid username or password".to_string(),
            user_id: None,
        })));
    }

    let session_id = uuid::Uuid::new_v4().to_string();
    let mut headers = axum::http::HeaderMap::new();

    let cookie = format!("session_id={}; Path=/; HttpOnly; SameSite=Strict; Max-Age=86400", session_id);
    headers.insert("Set-Cookie", cookie.parse().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?);

    match state.db.create_session(&session_id, user.id).await {
        Ok(_) => Ok((headers, Json(AuthResponse {
            success: true,
            message: "Login successful".to_string(),
            user_id: Some(user.id),
        }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn logout() -> Result<Json<AuthResponse>, StatusCode> {
    Ok(Json(AuthResponse {
        success: true,
        message: "Logged out successfully".to_string(),
        user_id: None,
    }))
}

pub async fn check_auth() -> Result<Json<AuthResponse>, StatusCode> {
    Ok(Json(AuthResponse {
        success: false,
        message: "Not authenticated".to_string(),
        user_id: None,
    }))
}


