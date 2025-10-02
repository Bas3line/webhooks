use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub bind_address: String,
    pub database_url: String,
    pub base_url: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let bind_address = env::var("BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0:5050".to_string());
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| anyhow::anyhow!("DATABASE_URL environment variable is required"))?;
        let base_url = env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:5050".to_string());

        Ok(Config {
            bind_address,
            database_url,
            base_url,
        })
    }
}