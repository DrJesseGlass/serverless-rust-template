use crate::{json_response, ApiResponse, AppState};
use aws_lambda_events::apigw::{ApiGatewayV2httpRequest, ApiGatewayV2httpResponse};
use aws_sdk_dynamodb::types::AttributeValue;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use shared::models::Item;
use tracing::{error, info};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateItemRequest {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

impl CreateItemRequest {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.name.is_empty() || self.name.len() > 256 {
            return Err("Name must be 1-256 characters");
        }
        if let Some(desc) = &self.description {
            if desc.len() > 4096 {
                return Err("Description must be under 4096 characters");
            }
        }
        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct ListItemsResponse {
    pub items: Vec<Item>,
    pub count: usize,
}

pub async fn list(state: &AppState, request: &ApiGatewayV2httpRequest) -> ApiGatewayV2httpResponse {
    let limit = request
        .query_string_parameters
        .first("limit")
        .and_then(|l| l.parse::<i32>().ok())
        .unwrap_or(50)
        .min(100);  // Cap at 100

    let result = state.dynamo.query()
        .table_name(&state.config.table_name)
        .key_condition_expression("pk = :pk")
        .expression_attribute_values(":pk", AttributeValue::S("ITEM".to_string()))
        .limit(limit)
        .send()
        .await;

    match result {
        Ok(output) => {
            let items: Vec<Item> = output
                .items
                .unwrap_or_default()
                .into_iter()
                .filter_map(|item| Item::from_dynamo(&item).ok())
                .collect();
            let count = items.len();
            info!(count = count, "Listed items");
            json_response(
                200,
                &ApiResponse::success(ListItemsResponse { items, count }),
            )
        }
        Err(e) => {
            error!(error = %e, "Failed to list items");
            json_response(500, &ApiResponse::<()>::error("Failed to list items"))
        }
    }
}

pub async fn create(
    state: &AppState,
    request: &ApiGatewayV2httpRequest,
) -> ApiGatewayV2httpResponse {
    let body = match &request.body {
        Some(body) => body,
        None => return json_response(400, &ApiResponse::<()>::error("Missing request body")),
    };

    let create_req: CreateItemRequest = match serde_json::from_str(body) {
        Ok(req) => req,
        Err(e) => return json_response(400, &ApiResponse::<()>::error(format!("Invalid JSON: {e}"))),
    };

    if let Err(e) = create_req.validate() {
        return json_response(400, &ApiResponse::<()>::error(e));
    }

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    let item = Item {
        id: id.clone(),
        name: create_req.name,
        description: create_req.description,
        created_at: now.clone(),
        updated_at: now,
    };

    let result = state
        .dynamo
        .put_item()
        .table_name(&state.config.table_name)
        .item("pk", AttributeValue::S("ITEM".to_string()))
        .item("sk", AttributeValue::S(format!("ITEM#{id}")))
        .item("id", AttributeValue::S(item.id.clone()))
        .item("name", AttributeValue::S(item.name.clone()))
        .item(
            "description",
            item.description
                .as_ref()
                .map(|d| AttributeValue::S(d.clone()))
                .unwrap_or(AttributeValue::Null(true)),
        )
        .item("created_at", AttributeValue::S(item.created_at.clone()))
        .item("updated_at", AttributeValue::S(item.updated_at.clone()))
        .item("gsi1pk", AttributeValue::S("ITEM".to_string()))
        .item("gsi1sk", AttributeValue::S(item.created_at.clone()))
        .send()
        .await;

    match result {
        Ok(_) => {
            info!(id = %id, "Created item");
            json_response(201, &ApiResponse::success(item))
        }
        Err(e) => {
            error!(error = %e, "Failed to create item");
            json_response(500, &ApiResponse::<()>::error("Failed to create item"))
        }
    }
}

pub async fn get(state: &AppState, request: &ApiGatewayV2httpRequest) -> ApiGatewayV2httpResponse {
    let path = request.raw_path.as_deref().unwrap_or("");
    let id = path.trim_start_matches("/items/");

    if id.is_empty() {
        return json_response(400, &ApiResponse::<()>::error("Missing item ID"));
    }

    let result = state
        .dynamo
        .get_item()
        .table_name(&state.config.table_name)
        .key("pk", AttributeValue::S("ITEM".to_string()))
        .key("sk", AttributeValue::S(format!("ITEM#{id}")))
        .send()
        .await;

    match result {
        Ok(output) => match output.item {
            Some(item) => match Item::from_dynamo(&item) {
                Ok(item) => json_response(200, &ApiResponse::success(item)),
                Err(e) => {
                    error!(error = %e, "Failed to parse item");
                    json_response(500, &ApiResponse::<()>::error("Failed to parse item"))
                }
            },
            None => json_response(404, &ApiResponse::<()>::error("Item not found")),
        },
        Err(e) => {
            error!(error = %e, "Failed to get item");
            json_response(500, &ApiResponse::<()>::error("Failed to get item"))
        }
    }
}

pub async fn delete(
    state: &AppState,
    request: &ApiGatewayV2httpRequest,
) -> ApiGatewayV2httpResponse {
    let path = request.raw_path.as_deref().unwrap_or("");
    let id = path.trim_start_matches("/items/");

    if id.is_empty() {
        return json_response(400, &ApiResponse::<()>::error("Missing item ID"));
    }

    let result = state
        .dynamo
        .delete_item()
        .table_name(&state.config.table_name)
        .key("pk", AttributeValue::S("ITEM".to_string()))
        .key("sk", AttributeValue::S(format!("ITEM#{id}")))
        .send()
        .await;

    match result {
        Ok(_) => {
            info!(id = %id, "Deleted item");
            json_response(204, &ApiResponse::success(()))
        }
        Err(e) => {
            error!(error = %e, "Failed to delete item");
            json_response(500, &ApiResponse::<()>::error("Failed to delete item"))
        }
    }
}
