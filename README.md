# Serverless Rust Template

A production-ready, full-stack serverless template with native mobile apps sharing a Rust core.

## Why This Architecture?

**Shared Rust Core:** Business logic is written once in Rust and compiled to:
- Native Android library (`.so` via NDK)
- Native iOS library (`.a` via Xcode)
- WebAssembly (for web, optional)
- Lambda functions (backend)

This eliminates code duplication across platforms while maintaining native performance.

**Serverless Backend:** AWS Lambda with Rust provides ~10-15ms cold starts and near-zero cost when idle—perfect for apps that need to scale from 0 to millions of users.

**UniFFI Bindings:** Mozilla's UniFFI generates type-safe bindings from Rust to Kotlin/Swift automatically. You define the interface once in a `.udl` file.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Shared Rust Core                         │
│              (Authentication, API client, business logic)       │
│                         core/src/lib.rs                         │
└─────────────────┬───────────────────────────┬───────────────────┘
                  │                           │
        ┌─────────▼─────────┐       ┌─────────▼─────────┐
        │   Android App     │       │     iOS App       │
        │  (Kotlin/Compose) │       │   (Swift/SwiftUI) │
        │   via UniFFI/JNI  │       │    via UniFFI     │
        └───────────────────┘       └───────────────────┘
                  │                           │
                  └───────────┬───────────────┘
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                         AWS Backend                             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────────┐ │
│  │ Cognito  │  │   API    │  │  Lambda  │  │    DynamoDB      │ │
│  │  (Auth)  │  │ Gateway  │  │  (Rust)  │  │    (Data)        │ │
│  └──────────┘  └──────────┘  └──────────┘  └──────────────────┘ │
│  ┌──────────┐  ┌──────────┐                                     │
│  │    S3    │  │CloudFront│  ← React Frontend                   │
│  │(Storage) │  │  (CDN)   │                                     │
│  └──────────┘  └──────────┘                                     │
└─────────────────────────────────────────────────────────────────┘
```

## Features

- 🦀 **Rust Lambda** — ~10-15ms cold starts, low memory, low cost
- 📱 **Native Mobile** — Android (Kotlin/Compose) + iOS (Swift/SwiftUI)
- 🔗 **Shared Core** — Write business logic once, run everywhere
- 🔐 **Cognito Auth** — Google OAuth built-in, extensible to other providers
- 🔑 **OIDC Authentication** — No AWS credentials stored in GitHub
- 📦 **Single-table DynamoDB** — Ready for complex access patterns
- ⚡ **Auto-scaling** — Handles 1 to 10,000+ concurrent users
- 💰 **Scales to zero** — Near-zero cost when idle

## Tech Stack

| Layer | Technology |
|-------|------------|
| **Web Frontend** | React + TypeScript + Tailwind |
| **Mobile (Android)** | Kotlin + Jetpack Compose |
| **Mobile (iOS)** | Swift + SwiftUI |
| **Shared Core** | Rust + UniFFI |
| **Backend** | Rust Lambda functions |
| **Database** | DynamoDB |
| **Auth** | AWS Cognito + Google OAuth |
| **Infrastructure** | Terraform |
| **CI/CD** | GitHub Actions with OIDC |

## Project Structure

```
.
├── .github/workflows/
│   └── deploy.yml              # CI/CD pipeline (OIDC auth)
├── core/                       # Shared Rust library (UniFFI)
│   ├── src/
│   │   ├── lib.rs              # Core logic & FFI exports
│   │   ├── myapp.udl           # UniFFI interface definition
│   │   └── bin/uniffi-bindgen.rs
│   ├── bindings/               # Generated bindings (Kotlin/Swift)
│   └── Cargo.toml
├── mobile/
│   └── android/                # Native Android app
│       ├── app/src/main/
│       │   ├── java/.../MainActivity.kt
│       │   ├── jniLibs/        # Compiled Rust .so files
│       │   └── AndroidManifest.xml
│       └── build.gradle.kts
├── frontend/                   # React + Vite + Tailwind
│   ├── src/
│   │   ├── App.tsx
│   │   └── auth/               # Auth context & components
│   └── package.json
├── lambdas/                    # Rust Lambda workspace
│   ├── api-handler/
│   │   └── src/
│   │       ├── main.rs
│   │       └── auth.rs         # JWT validation
│   └── shared/
│       └── src/
│           ├── config.rs
│           └── models.rs
└── infra/                      # Terraform
    ├── main.tf
    ├── backend.tf
    ├── auth.tf                 # Cognito + Google OAuth
    ├── api.tf
    ├── data.tf
    └── github-oidc.tf
```

---

## Quick Start

### Prerequisites

**All platforms:**
- [Rust](https://rustup.rs/)
- [cargo-lambda](https://www.cargo-lambda.info/guide/installation.html): `brew tap cargo-lambda/cargo-lambda && brew install cargo-lambda`
- [Node.js 18+](https://nodejs.org/)
- [Terraform 1.5+](https://www.terraform.io/downloads)
- [AWS CLI v2](https://aws.amazon.com/cli/)

**Android development:**
- [Android Studio](https://developer.android.com/studio) (for SDK & NDK)
- Java 17+: `brew install openjdk@17`
- Rust Android targets:
  ```bash
  rustup target add aarch64-linux-android armv7-linux-androideabi
  cargo install cargo-ndk
  ```
- Android NDK: Android Studio → Settings → SDK Tools → NDK (Side by side)

**iOS development:**
- Xcode 15+
- Rust iOS targets:
  ```bash
  rustup target add aarch64-apple-ios aarch64-apple-ios-sim
  ```

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

### 4. Set Up Google OAuth (Optional but Recommended)

1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Create a new project or select existing
3. Enable OAuth consent screen (External, add test users)
4. Create OAuth 2.0 credentials (Web application)
5. Note the Client ID and Client Secret

Add to GitHub Secrets:
- `GOOGLE_CLIENT_ID`
- `GOOGLE_CLIENT_SECRET`

### 5. First Deploy (Local)

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
terraform apply \
  -var="google_client_id=YOUR_GOOGLE_CLIENT_ID" \
  -var="google_client_secret=YOUR_GOOGLE_CLIENT_SECRET"
```

Save the outputs:
- `cloudfront_url` — Your frontend
- `api_url` — Your API
- `cognito_domain` — Auth endpoint
- `cognito_client_id` — For mobile apps
- `github_actions_role_arn` — For GitHub Actions

### 6. Configure GitHub Actions

Go to your repo → **Settings** → **Secrets and variables** → **Actions**

Add secrets:
- `AWS_ROLE_ARN` — `github_actions_role_arn` from Terraform output
- `GOOGLE_CLIENT_ID` — From Google Cloud Console
- `GOOGLE_CLIENT_SECRET` — From Google Cloud Console

### 7. Update Frontend & Mobile Config

**`frontend/.env.production`**
```
VITE_API_URL=https://xxxxx.execute-api.us-east-1.amazonaws.com
VITE_COGNITO_DOMAIN=https://myapp-dev-xxxxx.auth.us-east-1.amazoncognito.com
VITE_COGNITO_CLIENT_ID=xxxxx
```

**`mobile/android/app/src/main/java/.../MainActivity.kt`**
```kotlin
companion object {
    private const val API_URL = "https://xxxxx.execute-api.us-east-1.amazonaws.com"
    private const val COGNITO_DOMAIN = "https://myapp-dev-xxxxx.auth.us-east-1.amazoncognito.com"
    private const val COGNITO_CLIENT_ID = "xxxxx"
}
```

Commit and push—GitHub Actions will deploy automatically.

---

## Local Development

### Backend (Rust Lambda)

```bash
cd lambdas
cargo lambda watch

# Test endpoints
curl http://localhost:9000/lambda-url/api-handler/health
curl http://localhost:9000/lambda-url/api-handler/items
```

### Web Frontend (React)

```bash
cd frontend
npm install
npm run dev
# Opens http://localhost:5173
```

### Rust Core (Shared Library)

```bash
cd core
cargo build
cargo test
```

### Android App

#### First-time setup

1. Set NDK path:
   ```bash
   export ANDROID_NDK_HOME=~/Library/Android/sdk/ndk/<version>
   ```

2. Build Rust core for Android:
   ```bash
   cd core
   cargo ndk -t arm64-v8a -t armeabi-v7a build --release
   ```

3. Generate Kotlin bindings:
   ```bash
   cargo run --features cli --bin uniffi-bindgen generate \
     --library target/aarch64-linux-android/release/libmyapp.so \
     --language kotlin --out-dir ./bindings
   ```

4. Copy artifacts to Android project:
   ```bash
   cp bindings/uniffi/myapp/myapp.kt \
     ../mobile/android/app/src/main/java/uniffi/myapp/
   cp target/aarch64-linux-android/release/libmyapp.so \
     ../mobile/android/app/src/main/jniLibs/arm64-v8a/
   cp target/armv7-linux-androideabi/release/libmyapp.so \
     ../mobile/android/app/src/main/jniLibs/armeabi-v7a/
   ```

#### Build & Run

Connect an Android device with USB debugging enabled:

```bash
cd mobile/android

# Build debug APK
./gradlew assembleDebug

# Build and install on connected device
./gradlew installDebug

# View logs
~/Library/Android/sdk/platform-tools/adb logcat | grep -E "(Myapp|AndroidRuntime)"
```

#### After Rust Core Changes

```bash
# Rebuild script (run from project root)
cd core
cargo ndk -t arm64-v8a -t armeabi-v7a build --release
cargo run --features cli --bin uniffi-bindgen generate \
  --library target/aarch64-linux-android/release/libmyapp.so \
  --language kotlin --out-dir ./bindings
cp bindings/uniffi/myapp/myapp.kt \
  ../mobile/android/app/src/main/java/uniffi/myapp/
cp target/aarch64-linux-android/release/libmyapp.so \
  ../mobile/android/app/src/main/jniLibs/arm64-v8a/
cp target/armv7-linux-androideabi/release/libmyapp.so \
  ../mobile/android/app/src/main/jniLibs/armeabi-v7a/
cd ../mobile/android
./gradlew installDebug
```

---

## Authentication Flow

### Mobile (OAuth with PKCE)

```
┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐
│  Mobile  │     │ Browser  │     │ Cognito  │     │  Google  │
│   App    │     │ (Custom  │     │          │     │          │
│          │     │   Tab)   │     │          │     │          │
└────┬─────┘     └────┬─────┘     └────┬─────┘     └────┬─────┘
     │                │                │                │
     │ 1. Open auth URL               │                │
     │───────────────>│                │                │
     │                │ 2. Redirect    │                │
     │                │───────────────>│                │
     │                │                │ 3. Google OAuth│
     │                │                │───────────────>│
     │                │                │<───────────────│
     │                │ 4. Redirect with code           │
     │<───────────────│ (myapp://auth?code=xxx)         │
     │                │                │                │
     │ 5. Exchange code for tokens    │                │
     │────────────────────────────────>│                │
     │<────────────────────────────────│                │
     │ (access_token, id_token)        │                │
     │                │                │                │
     │ 6. Store tokens in Rust core   │                │
     │ 7. Parse user from JWT          │                │
```

### Web (Similar flow with redirect)

The web app uses the same Cognito endpoints but redirects to the web URL instead of a custom scheme.

---

## Rust Core (UniFFI)

The `core/` crate provides shared functionality across all platforms.

### Exported Functions

| Function | Description |
|----------|-------------|
| `initialize(config)` | Set API and Cognito configuration |
| `set_auth_tokens(tokens)` | Store tokens after OAuth flow |
| `clear_auth()` | Clear stored tokens (logout) |
| `is_authenticated()` | Check if valid tokens exist |
| `get_current_user()` | Parse user info from stored ID token |
| `get_auth_url(redirect)` | Build OAuth authorization URL |
| `get_token_endpoint()` | Get Cognito token endpoint URL |
| `get_api_url()` | Get configured API base URL |
| `get_access_token()` | Get token for authenticated API calls |

### Adding New Functions

1. **Add Rust function** in `core/src/lib.rs`:
   ```rust
   #[uniffi::export]
   pub fn my_new_function(input: String) -> Result<String, CoreError> {
       // Implementation
       Ok(format!("Processed: {}", input))
   }
   ```

2. **Update UDL** in `core/src/myapp.udl`:
   ```
   namespace myapp {
     // ... existing functions ...
     [Throws=CoreError]
     string my_new_function(string input);
   };
   ```

3. **Rebuild bindings** (see "After Rust Core Changes" above)

4. **Use in Kotlin**:
   ```kotlin
   import uniffi.myapp.*
   
   val result = myNewFunction("hello")
   ```

5. **Use in Swift**:
   ```swift
   import Myapp
   
   let result = try myNewFunction(input: "hello")
   ```

### Error Handling

Define errors in Rust:
```rust
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum CoreError {
    #[error("Not authenticated")]
    NotAuthenticated,
    #[error("Network error: {msg}")]
    Network { msg: String },
}
```

> ⚠️ **Note:** Avoid using `message` as a field name—it conflicts with Kotlin's `Throwable.message`. Use `msg` instead.

---

## Adding Mobile OAuth Redirect

When setting up a new project, add your mobile redirect URI to Cognito:

**`infra/auth.tf`**
```hcl
callback_urls = [
  "https://${aws_cloudfront_distribution.frontend.domain_name}",
  "http://localhost:5173",
  "myapp://auth"  # Mobile deep link
]

logout_urls = [
  "https://${aws_cloudfront_distribution.frontend.domain_name}",
  "http://localhost:5173",
  "myapp://auth"
]
```

The mobile app's `AndroidManifest.xml` registers to handle `myapp://auth`:
```xml
<intent-filter>
    <action android:name="android.intent.action.VIEW" />
    <category android:name="android.intent.category.DEFAULT" />
    <category android:name="android.intent.category.BROWSABLE" />
    <data android:scheme="myapp" android:host="auth" />
</intent-filter>
```

---

## Deployment

### Automatic (GitHub Actions)

Push to `main` triggers automatic deployment of:
- ✅ Lambda functions
- ✅ Web frontend
- ✅ Infrastructure changes

Mobile apps require manual deployment to app stores.

### Manual Android Release

```bash
cd mobile/android

# Build release APK (requires signing config)
./gradlew assembleRelease

# Output: app/build/outputs/apk/release/app-release.apk
```

---

## Scaling

| Users | Lambda Instances | Est. Monthly Cost |
|-------|------------------|-------------------|
| 10    | 1-2              | < $1              |
| 100   | 5-10             | $5-10             |
| 1,000 | 50-100           | $50-100           |
| 10,000| 500-1000         | $200-500          |

Cognito pricing: First 50,000 MAU free, then $0.0055/user.

---

## Troubleshooting

### Android: "Library not found"
Ensure the `.so` files are in the correct jniLibs folders and the library name in Cargo.toml matches what UniFFI expects.

### Android: "UniFFI API checksum mismatch"
The Kotlin bindings don't match the compiled library. Regenerate bindings from the same library build:
```bash
cargo run --features cli --bin uniffi-bindgen generate \
  --library target/aarch64-linux-android/release/libmyapp.so \
  --language kotlin --out-dir ./bindings
```

### Android: "No connected devices"
Enable USB debugging: Settings → Developer options → USB debugging. You may need to revoke and re-authorize.

### Cognito: "redirect_mismatch"
Add the redirect URI to both `callback_urls` and `logout_urls` in `infra/auth.tf`, then `terraform apply`.

---

## License

MIT
