use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub table_name: String,
    pub storage_bucket: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            table_name: env::var("TABLE_NAME").unwrap_or_else(|_| "items".to_string()),
            storage_bucket: env::var("STORAGE_BUCKET").unwrap_or_else(|_| "storage".to_string()),
        }
    }
}
