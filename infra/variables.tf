variable "aws_region" {
  description = "AWS region for resources"
  type        = string
  default     = "us-east-1"
}

variable "project_name" {
  description = "Project name used for resource naming"
  type        = string
  default     = "myapp"
}

variable "environment" {
  description = "Environment (dev, staging, prod)"
  type        = string
  default     = "dev"
}

variable "lambda_memory_size" {
  description = "Memory size for Lambda functions in MB"
  type        = number
  default     = 256
}

variable "lambda_timeout" {
  description = "Timeout for Lambda functions in seconds"
  type        = number
  default     = 30
}

variable "github_repo" {
  description = "GitHub repository in format: owner/repo"
  type        = string
  # Example: "YOUR_USERNAME/YOUR_REPO"
}

variable "terraform_state_bucket" {
  description = "S3 bucket name for Terraform state"
  type        = string
}
