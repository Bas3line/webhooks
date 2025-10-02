use sqlx::{PgPool, Row};
use anyhow::Result;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;

use crate::{models::*};

#[derive(Debug, Clone)]
pub struct Database {
    pub pool: PgPool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPool::connect(database_url).await?;
        sqlx::migrate!("./migrations").run(&pool).await?;
        Ok(Self { pool })
    }

    pub async fn create_user(&self, username: &str, password_hash: &str) -> Result<User> {
        let row = sqlx::query(
            "INSERT INTO users (username, password_hash) VALUES ($1, $2) RETURNING id, username, password_hash, created_at, updated_at, is_active"
        )
        .bind(username)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await?;

        Ok(User {
            id: row.get("id"),
            username: row.get("username"),
            password_hash: row.get("password_hash"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            is_active: row.get("is_active"),
        })
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let row = sqlx::query(
            "SELECT id, username, password_hash, created_at, updated_at, is_active FROM users WHERE username = $1"
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(User {
                id: row.get("id"),
                username: row.get("username"),
                password_hash: row.get("password_hash"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                is_active: row.get("is_active"),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_user_by_id(&self, user_id: Uuid) -> Result<Option<User>> {
        let row = sqlx::query(
            "SELECT id, username, password_hash, created_at, updated_at, is_active FROM users WHERE id = $1"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(User {
                id: row.get("id"),
                username: row.get("username"),
                password_hash: row.get("password_hash"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                is_active: row.get("is_active"),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn create_webhook(&self, user_id: Option<Uuid>, name: &str, endpoint: &str, secret: &str, description: Option<&str>) -> Result<Webhook> {
        let row = sqlx::query(
            "INSERT INTO webhooks (user_id, name, endpoint, secret, description) VALUES ($1, $2, $3, $4, $5) RETURNING id, user_id, name, endpoint, secret, description, is_active, created_at, updated_at"
        )
        .bind(user_id)
        .bind(name)
        .bind(endpoint)
        .bind(secret)
        .bind(description)
        .fetch_one(&self.pool)
        .await?;

        Ok(Webhook {
            id: row.get("id"),
            user_id: row.get("user_id"),
            name: row.get("name"),
            endpoint: row.get("endpoint"),
            secret: row.get("secret"),
            description: row.get("description"),
            is_active: row.get("is_active"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    pub async fn get_webhooks_by_user(&self, user_id: Uuid) -> Result<Vec<Webhook>> {
        let rows = sqlx::query(
            "SELECT id, user_id, name, endpoint, secret, description, is_active, created_at, updated_at FROM webhooks WHERE user_id = $1 ORDER BY created_at DESC"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        let mut webhooks = Vec::new();
        for row in rows {
            webhooks.push(Webhook {
                id: row.get("id"),
                user_id: row.get("user_id"),
                name: row.get("name"),
                endpoint: row.get("endpoint"),
                secret: row.get("secret"),
                description: row.get("description"),
                is_active: row.get("is_active"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }

        Ok(webhooks)
    }

    pub async fn get_webhook(&self, id: Uuid, user_id: Uuid) -> Result<Option<Webhook>> {
        let row = sqlx::query(
            "SELECT id, user_id, name, endpoint, secret, description, is_active, created_at, updated_at FROM webhooks WHERE id = $1 AND user_id = $2"
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(Webhook {
                id: row.get("id"),
                user_id: row.get("user_id"),
                name: row.get("name"),
                endpoint: row.get("endpoint"),
                secret: row.get("secret"),
                description: row.get("description"),
                is_active: row.get("is_active"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_webhook_by_endpoint(&self, endpoint: &str) -> Result<Option<Webhook>> {
        let row = sqlx::query(
            "SELECT id, user_id, name, endpoint, secret, description, is_active, created_at, updated_at FROM webhooks WHERE endpoint = $1 AND is_active = true"
        )
        .bind(endpoint)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(Webhook {
                id: row.get("id"),
                user_id: row.get("user_id"),
                name: row.get("name"),
                endpoint: row.get("endpoint"),
                secret: row.get("secret"),
                description: row.get("description"),
                is_active: row.get("is_active"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn store_event(&self, event: &WebhookEvent) -> Result<()> {
        let headers_json = serde_json::to_value(&event.headers)?;

        sqlx::query(
            "INSERT INTO webhook_events (id, user_id, webhook_id, timestamp, headers, body, source_ip, endpoint, event_type, status_code) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
        )
        .bind(event.id)
        .bind(event.user_id)
        .bind(event.webhook_id)
        .bind(event.timestamp)
        .bind(headers_json)
        .bind(&event.body)
        .bind(&event.source_ip)
        .bind(&event.endpoint)
        .bind(&event.event_type)
        .bind(event.status_code)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_all_events(&self, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<WebhookEvent>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);

        let rows = sqlx::query(
            "SELECT id, user_id, webhook_id, timestamp, headers, body, source_ip, endpoint, event_type, status_code, created_at FROM webhook_events ORDER BY timestamp DESC LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let mut events = Vec::new();
        for row in rows {
            let headers: HashMap<String, String> = serde_json::from_value(row.get("headers"))?;

            events.push(WebhookEvent {
                id: row.get("id"),
                user_id: row.get("user_id"),
                webhook_id: row.get("webhook_id"),
                timestamp: row.get("timestamp"),
                headers,
                body: row.get("body"),
                source_ip: row.get("source_ip"),
                endpoint: row.get("endpoint"),
                event_type: row.get("event_type"),
                status_code: row.get("status_code"),
                created_at: row.get("created_at"),
            });
        }

        Ok(events)
    }

    pub async fn get_events_by_user(&self, user_id: Uuid, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<WebhookEvent>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);

        let rows = sqlx::query(
            "SELECT id, user_id, webhook_id, timestamp, headers, body, source_ip, endpoint, event_type, status_code, created_at FROM webhook_events WHERE user_id = $1 ORDER BY timestamp DESC LIMIT $2 OFFSET $3"
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let mut events = Vec::new();
        for row in rows {
            let headers: HashMap<String, String> = serde_json::from_value(row.get("headers"))?;

            events.push(WebhookEvent {
                id: row.get("id"),
                user_id: row.get("user_id"),
                webhook_id: row.get("webhook_id"),
                timestamp: row.get("timestamp"),
                headers,
                body: row.get("body"),
                source_ip: row.get("source_ip"),
                endpoint: row.get("endpoint"),
                event_type: row.get("event_type"),
                status_code: row.get("status_code"),
                created_at: row.get("created_at"),
            });
        }

        Ok(events)
    }

    pub async fn get_webhook_event_count(&self, webhook_id: Uuid, user_id: Uuid) -> Result<i64> {
        let row = sqlx::query(
            "SELECT COUNT(*) as count FROM webhook_events WHERE webhook_id = $1 AND user_id = $2"
        )
        .bind(webhook_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.get::<i64, _>("count"))
    }

    pub async fn update_webhook(&self, id: Uuid, user_id: Uuid, name: Option<&str>, description: Option<&str>, is_active: Option<bool>) -> Result<Option<Webhook>> {
        let name_update = name.unwrap_or("");
        let desc_update = description.unwrap_or("");
        let active_update = is_active.unwrap_or(true);

        let row = sqlx::query(
            "UPDATE webhooks SET name = COALESCE(NULLIF($3, ''), name), description = CASE WHEN $4 = '' THEN description ELSE $4 END, is_active = $5, updated_at = NOW() WHERE id = $1 AND user_id = $2 RETURNING id, user_id, name, endpoint, secret, description, is_active, created_at, updated_at"
        )
        .bind(id)
        .bind(user_id)
        .bind(name_update)
        .bind(desc_update)
        .bind(active_update)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(Webhook {
                id: row.get("id"),
                user_id: row.get("user_id"),
                name: row.get("name"),
                endpoint: row.get("endpoint"),
                secret: row.get("secret"),
                description: row.get("description"),
                is_active: row.get("is_active"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_events_by_webhook(&self, webhook_id: Uuid, user_id: Uuid, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<WebhookEvent>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);

        let rows = sqlx::query(
            "SELECT id, user_id, webhook_id, timestamp, headers, body, source_ip, endpoint, event_type, status_code, created_at FROM webhook_events WHERE webhook_id = $1 AND user_id = $2 ORDER BY timestamp DESC LIMIT $3 OFFSET $4"
        )
        .bind(webhook_id)
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let mut events = Vec::new();
        for row in rows {
            let headers: HashMap<String, String> = serde_json::from_value(row.get("headers"))?;

            events.push(WebhookEvent {
                id: row.get("id"),
                user_id: row.get("user_id"),
                webhook_id: row.get("webhook_id"),
                timestamp: row.get("timestamp"),
                headers,
                body: row.get("body"),
                source_ip: row.get("source_ip"),
                endpoint: row.get("endpoint"),
                event_type: row.get("event_type"),
                status_code: row.get("status_code"),
                created_at: row.get("created_at"),
            });
        }

        Ok(events)
    }

    pub async fn get_event(&self, id: Uuid, user_id: Uuid) -> Result<Option<WebhookEvent>> {
        let row = sqlx::query(
            "SELECT id, user_id, webhook_id, timestamp, headers, body, source_ip, endpoint, event_type, status_code, created_at FROM webhook_events WHERE id = $1 AND user_id = $2"
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let headers: HashMap<String, String> = serde_json::from_value(row.get("headers"))?;

            Ok(Some(WebhookEvent {
                id: row.get("id"),
                user_id: row.get("user_id"),
                webhook_id: row.get("webhook_id"),
                timestamp: row.get("timestamp"),
                headers,
                body: row.get("body"),
                source_ip: row.get("source_ip"),
                endpoint: row.get("endpoint"),
                event_type: row.get("event_type"),
                status_code: row.get("status_code"),
                created_at: row.get("created_at"),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn update_webhook_secret(&self, id: Uuid, user_id: Uuid, secret: &str) -> Result<Option<Webhook>> {
        let row = sqlx::query(
            "UPDATE webhooks SET secret = $3, updated_at = NOW() WHERE id = $1 AND user_id = $2 RETURNING id, user_id, name, endpoint, secret, description, is_active, created_at, updated_at"
        )
        .bind(id)
        .bind(user_id)
        .bind(secret)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(Webhook {
                id: row.get("id"),
                user_id: row.get("user_id"),
                name: row.get("name"),
                endpoint: row.get("endpoint"),
                secret: row.get("secret"),
                description: row.get("description"),
                is_active: row.get("is_active"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn delete_webhook(&self, id: Uuid, user_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM webhooks WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn create_session(&self, session_id: &str, user_id: Uuid) -> Result<()> {
        let expiry = chrono::Utc::now() + chrono::Duration::days(1);
        sqlx::query("INSERT INTO user_sessions (id, user_id, data, expiry_date) VALUES ($1, $2, $3, $4)")
            .bind(session_id)
            .bind(user_id)
            .bind(&[0u8]) // Empty data for now
            .bind(expiry)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_session_user(&self, session_id: &str) -> Result<Option<Uuid>> {
        let row = sqlx::query("SELECT user_id FROM user_sessions WHERE id = $1 AND expiry_date > NOW()")
            .bind(session_id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            Ok(Some(row.get("user_id")))
        } else {
            Ok(None)
        }
    }

    pub async fn delete_session(&self, session_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM user_sessions WHERE id = $1")
            .bind(session_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}