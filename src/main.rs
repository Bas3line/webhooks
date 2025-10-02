use axum::{
    extract::connect_info::ConnectInfo,
    http::StatusCode,
    response::{Html, Json},
    routing::{get, post, put, delete},
    Router,
};
use chrono::Utc;
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer, services::ServeDir};
use tracing::info;

mod config;
mod handlers;
mod database;
mod models;
mod api;
mod middleware;
mod auth;

use config::Config;
use database::Database;

#[derive(Debug, Clone)]
pub struct AppState {
    pub config: Config,
    pub db: Database,
}

impl AppState {
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        let db = Database::new(&config.database_url).await?;
        Ok(Self { config, db })
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = Config::from_env()?;
    let state = AppState::new(config.clone()).await?;

    let protected_routes = Router::new()
        .route("/api/webhooks", get(api::list_webhooks))
        .route("/api/webhooks", post(api::create_webhook))
        .route("/api/webhooks/:id", get(api::get_webhook))
        .route("/api/webhooks/:id", put(api::update_webhook))
        .route("/api/webhooks/:id", delete(api::delete_webhook))
        .route("/api/webhooks/:id/regenerate-secret", post(api::regenerate_secret))
        .route("/api/events", get(api::list_events))
        .route("/api/events/:id", get(api::get_event))
;

    let app = Router::new()
        .route("/", get(serve_dashboard))
        .route("/login", get(serve_login))
        .route("/events", get(serve_events_page))
        .route("/health", get(health_check))
        .route("/api/register", post(auth::register))
        .route("/api/login", post(auth::login))
        .route("/api/logout", post(auth::logout))
        .route("/api/auth", get(auth::check_auth))
        .merge(protected_routes)
        .route("/webhook/:endpoint", post(handlers::receive_webhook))
        .route("/webhook", post(handlers::receive_generic_webhook))
        .nest_service("/static", ServeDir::new("static"))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&config.bind_address).await?;
    info!("Webhook service listening on {}", config.bind_address);
    info!("Frontend available at http://{}", config.bind_address);

    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await?;
    Ok(())
}

async fn serve_dashboard() -> Html<String> {
    match tokio::fs::read_to_string("static/index.html").await {
        Ok(content) => Html(content),
        Err(_) => Html("<h1>Dashboard not found</h1>".to_string()),
    }
}

async fn serve_login() -> Result<Html<String>, StatusCode> {
    match tokio::fs::read_to_string("static/login.html").await {
        Ok(content) => Ok(Html(content)),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

async fn serve_events_page() -> Result<Html<String>, StatusCode> {
    match tokio::fs::read_to_string("static/events.html").await {
        Ok(content) => Ok(Html(content)),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": Utc::now()
    }))
}