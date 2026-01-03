use crate::{json_response, ApiResponse, AppState};
use aws_lambda_events::apigw::ApiGatewayV2httpResponse;
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

pub async fn handle(_state: &AppState) -> ApiGatewayV2httpResponse {
    json_response(
        200,
        &ApiResponse::success(HealthResponse {
            status: "healthy".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }),
    )
}
