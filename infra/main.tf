terraform {
  required_version = ">= 1.5.0"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

provider "aws" {
  region = var.aws_region

  default_tags {
    tags = {
      Project     = var.project_name
      Environment = var.environment
      ManagedBy   = "terraform"
    }
  }
}

# CloudFront requires ACM certs in us-east-1
provider "aws" {
  alias  = "us_east_1"
  region = "us-east-1"
}

locals {
  prefix = "${var.project_name}-${var.environment}"
}
