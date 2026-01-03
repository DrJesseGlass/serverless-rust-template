# Remote state configuration

terraform {
  backend "s3" {
    bucket = "REPLACE-WITH-YOUR-TFSTATE-BUCKET"
    key    = "myapp/terraform.tfstate"
    region = "us-east-1"
  }
}
