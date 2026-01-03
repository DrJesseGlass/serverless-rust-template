use aws_sdk_dynamodb::types::AttributeValue;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ModelError {
    #[error("Missing required attribute: {0}")]
    MissingAttribute(String),
    #[error("Invalid attribute type for {0}")]
    InvalidType(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl Item {
    pub fn from_dynamo(attrs: &HashMap<String, AttributeValue>) -> Result<Self, ModelError> {
        Ok(Self {
            id: get_string(attrs, "id")?,
            name: get_string(attrs, "name")?,
            description: get_optional_string(attrs, "description"),
            created_at: get_string(attrs, "created_at")?,
            updated_at: get_string(attrs, "updated_at")?,
        })
    }
}

fn get_string(attrs: &HashMap<String, AttributeValue>, key: &str) -> Result<String, ModelError> {
    attrs
        .get(key)
        .and_then(|v| v.as_s().ok())
        .map(|s| s.to_string())
        .ok_or_else(|| ModelError::MissingAttribute(key.to_string()))
}

fn get_optional_string(attrs: &HashMap<String, AttributeValue>, key: &str) -> Option<String> {
    attrs
        .get(key)
        .and_then(|v| v.as_s().ok())
        .map(|s| s.to_string())
}
