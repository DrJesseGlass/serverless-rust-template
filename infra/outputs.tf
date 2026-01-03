output "cloudfront_url" {
  description = "CloudFront distribution URL for frontend"
  value       = "https://${aws_cloudfront_distribution.frontend.domain_name}"
}

output "cloudfront_distribution_id" {
  description = "CloudFront distribution ID (for cache invalidation)"
  value       = aws_cloudfront_distribution.frontend.id
}

output "api_url" {
  description = "API Gateway endpoint URL"
  value       = aws_apigatewayv2_stage.main.invoke_url
}

output "frontend_bucket" {
  description = "S3 bucket name for frontend files"
  value       = aws_s3_bucket.frontend.bucket
}

output "storage_bucket" {
  description = "S3 bucket name for file storage"
  value       = aws_s3_bucket.storage.bucket
}

output "dynamodb_table" {
  description = "DynamoDB table name"
  value       = aws_dynamodb_table.main.name
}

output "lambda_function_name" {
  description = "Lambda function name"
  value       = aws_lambda_function.api.function_name
}
