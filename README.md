# Serverless Rust Template

A production-ready, full-stack serverless template with:

- **Frontend:** React + TypeScript + Tailwind → S3 + CloudFront
- **Backend:** Rust Lambdas (ARM64) → API Gateway
- **Database:** DynamoDB (pay-per-request, scales to zero)
- **Storage:** S3
- **IaC:** Terraform
- **CI/CD:** GitHub Actions with OIDC (no stored AWS secrets!)

## Features

- 🦀 **Rust Lambda** — ~10-15ms cold starts, low memory, low cost
- 🔐 **OIDC Authentication** — No AWS credentials stored in GitHub
- 📦 **Single-table DynamoDB** — Ready for complex access patterns
- ⚡ **Auto-scaling** — Handles 1 to 10,000+ concurrent users
- 💰 **Scales to zero** — Near-zero cost when idle

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/)
- [cargo-lambda](https://www.cargo-lambda.info/guide/installation.html): `brew tap cargo-lambda/cargo-lambda && brew install cargo-lambda`
- [Node.js 18+](https://nodejs.org/)
- [Terraform 1.5+](https://www.terraform.io/downloads)
- [AWS CLI v2](https://aws.amazon.com/cli/)

### 1. Use This Template

Click **"Use this template"** → **"Create a new repository"** on GitHub.

Clone your new repo:

```bash
git clone git@github.com:YOUR_USERNAME/YOUR_REPO.git
cd YOUR_REPO
```

### 2. Create AWS Resources

```bash
# Configure AWS CLI (if not already done)
aws configure

# Create Terraform state bucket (name must be globally unique)
aws s3 mb s3://YOUR-PROJECT-tfstate-$(date +%s) --region us-east-1
aws s3api put-bucket-versioning \
  --bucket YOUR-PROJECT-tfstate-XXXXX \
  --versioning-configuration Status=Enabled
```

### 3. Configure the Project

Edit these files with your values:

**`infra/backend.tf`**
```hcl
bucket = "YOUR-PROJECT-tfstate-XXXXX"  # Your bucket from step 2
key    = "myapp/terraform.tfstate"
```

**`infra/variables.tf`**
```hcl
variable "project_name" {
  default = "myapp"  # Change to your project name
}
```

**`.github/workflows/deploy.yml`** (two places in Terraform steps)
```yaml
-var="github_repo=YOUR_USERNAME/YOUR_REPO"
-var="terraform_state_bucket=YOUR-PROJECT-tfstate-XXXXX"
```

### 4. First Deploy (Local)

```bash
# Build Lambda
cd lambdas
cargo lambda build --release --arm64
cd target/lambda/api-handler
zip bootstrap.zip bootstrap
cd ../../../..

# Deploy infrastructure
cd infra
terraform init
terraform apply
```

Save the outputs:
- `cloudfront_url` — Your frontend
- `api_url` — Your API
- `github_actions_role_arn` — For GitHub Actions

### 5. Configure GitHub Actions

1. Go to your repo → **Settings** → **Secrets and variables** → **Actions**
2. Add secret:
   - Name: `AWS_ROLE_ARN`
   - Value: `github_actions_role_arn` from Terraform output

### 6. Update Frontend API URL

Edit `frontend/.env.production`:
```
VITE_API_URL=https://xxxxx.execute-api.us-east-1.amazonaws.com
```

Commit and push—GitHub Actions will deploy automatically.

---

## Local Development

### Backend

```bash
cd lambdas
cargo lambda watch

# Test
curl http://localhost:9000/lambda-url/api-handler/health
```

### Frontend

```bash
cd frontend
npm install
npm run dev
```

---

## Project Structure

```
.
├── .github/workflows/
│   └── deploy.yml          # CI/CD pipeline (OIDC auth)
├── frontend/               # React + Vite + Tailwind
│   ├── src/
│   │   ├── App.tsx         # Main component
│   │   └── main.tsx
│   └── package.json
├── infra/                  # Terraform
│   ├── main.tf             # Provider config
│   ├── backend.tf          # S3 state backend
│   ├── variables.tf        # Configuration variables
│   ├── outputs.tf          # Output values
│   ├── frontend.tf         # S3 + CloudFront
│   ├── api.tf              # Lambda + API Gateway
│   ├── data.tf             # DynamoDB + S3 storage
│   ├── lambda-role.tf      # Lambda IAM role
│   └── github-oidc.tf      # GitHub Actions OIDC
└── lambdas/                # Rust workspace
    ├── Cargo.toml
    ├── api-handler/        # Main API Lambda
    │   └── src/
    │       ├── main.rs     # Router + handlers
    │       └── routes/
    └── shared/             # Shared library
        └── src/
            ├── config.rs   # Environment config
            └── models.rs   # Data models
```

---

## Architecture

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  CloudFront │────▶│     S3      │     │  DynamoDB   │
│    (CDN)    │     │  (Frontend) │     │  (Database) │
└─────────────┘     └─────────────┘     └──────▲──────┘
                                               │
┌─────────────┐     ┌─────────────┐     ┌──────┴──────┐
│   Browser   │────▶│ API Gateway │────▶│   Lambda    │
│             │     │   (HTTP)    │     │   (Rust)    │
└─────────────┘     └─────────────┘     └──────┬──────┘
                                               │
                                        ┌──────▼──────┐
                                        │     S3      │
                                        │  (Storage)  │
                                        └─────────────┘
```

---

## Scaling

| Users | Lambda Instances | Est. Monthly Cost |
|-------|------------------|-------------------|
| 10    | 1-2              | < $1              |
| 100   | 5-10             | $5-10             |
| 1,000 | 50-100           | $50-100           |
| 10,000| 500-1000         | $200-500          |

Default Lambda concurrency limit: 1,000 (can request increase).

---

## Customization

### Adding Routes

Edit `lambdas/api-handler/src/main.rs`:

```rust
let response = match (method, path) {
    ("GET", "/health") => routes::health::handle(state).await,
    ("GET", "/items") => routes::items::list(state, &request).await,
    ("POST", "/items") => routes::items::create(state, &request).await,
    // Add new routes here
    ("GET", "/users") => routes::users::list(state, &request).await,
    _ => json_response(404, &ApiResponse::<()>::error("Not found")),
};
```

### Adding Models

Edit `lambdas/shared/src/models.rs` and create corresponding route handlers.

### Environment Variables

Add to `infra/api.tf` in the Lambda environment block:

```hcl
environment {
  variables = {
    RUST_LOG       = "info"
    TABLE_NAME     = aws_dynamodb_table.main.name
    STORAGE_BUCKET = aws_s3_bucket.storage.bucket
    # Add new vars here
    MY_NEW_VAR     = "value"
  }
}
```

---

## License

MIT
