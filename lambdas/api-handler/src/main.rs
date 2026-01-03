use aws_lambda_events::apigw::{ApiGatewayV2httpRequest, ApiGatewayV2httpResponse};
use aws_lambda_events::encodings::Body;
use aws_lambda_events::http::HeaderMap;
use aws_sdk_dynamodb::Client as DynamoClient;
use aws_sdk_s3::Client as S3Client;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use shared::config::AppConfig;
use tracing::{info, instrument};

mod routes;

pub struct AppState {
    pub dynamo: DynamoClient,
    pub s3: S3Client,
    pub config: AppConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: impl Into<String>) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            data: None,
            error: Some(message.into()),
        }
    }
}

pub fn json_response<T: Serialize>(
    status_code: i64,
    body: &ApiResponse<T>,
) -> ApiGatewayV2httpResponse {
    let mut headers = HeaderMap::new();
    headers.insert("content-type", "application/json".parse().unwrap());
    headers.insert("access-control-allow-origin", "*".parse().unwrap());
    headers.insert(
        "access-control-allow-methods",
        "GET, POST, PUT, DELETE, OPTIONS".parse().unwrap(),
    );
    headers.insert(
        "access-control-allow-headers",
        "Content-Type, Authorization".parse().unwrap(),
    );

    ApiGatewayV2httpResponse {
        status_code,
        headers,
        multi_value_headers: HeaderMap::new(),
        body: Some(Body::Text(serde_json::to_string(body).unwrap_or_default())),
        is_base64_encoded: false,
        cookies: vec![],
    }
}

#[instrument(skip(state, event), fields(path = %event.payload.raw_path.as_deref().unwrap_or("/")))]
async fn router(
    state: &AppState,
    event: LambdaEvent<ApiGatewayV2httpRequest>,
) -> Result<ApiGatewayV2httpResponse, Error> {
    let request = event.payload;
    let method = request.request_context.http.method.as_str();
    let path = request.raw_path.as_deref().unwrap_or("/");

    info!(method = %method, path = %path, "Handling request");

    let response = match (method, path) {
        ("OPTIONS", _) => {
            let mut headers = HeaderMap::new();
            headers.insert("access-control-allow-origin", "*".parse().unwrap());
            headers.insert(
                "access-control-allow-methods",
                "GET, POST, PUT, DELETE, OPTIONS".parse().unwrap(),
            );
            headers.insert(
                "access-control-allow-headers",
                "Content-Type, Authorization".parse().unwrap(),
            );
            ApiGatewayV2httpResponse {
                status_code: 200,
                headers,
                multi_value_headers: HeaderMap::new(),
                body: None,
                is_base64_encoded: false,
                cookies: vec![],
            }
        }
        ("GET", "/health") => routes::health::handle(state).await,
        ("GET", "/items") => routes::items::list(state, &request).await,
        ("POST", "/items") => routes::items::create(state, &request).await,
        ("GET", p) if p.starts_with("/items/") => routes::items::get(state, &request).await,
        ("DELETE", p) if p.starts_with("/items/") => routes::items::delete(state, &request).await,
        _ => json_response(404, &ApiResponse::<()>::error("Not found")),
    };

    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("info".parse().unwrap()),
        )
        .without_time()
        .init();

    let aws_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .load()
        .await;
    let dynamo = DynamoClient::new(&aws_config);
    let s3 = S3Client::new(&aws_config);
    let config = AppConfig::from_env();

    info!(table = %config.table_name, bucket = %config.storage_bucket, "Starting Lambda");

    let state = AppState { dynamo, s3, config };
    lambda_runtime::run(service_fn(|event| router(&state, event))).await
}
