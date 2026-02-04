# Phase 09: GitHub Authentication

## Context Links
- [Plan Overview](./plan.md)
- [Phase 02: Core Architecture](./phase-02-core-architecture.md)
- [Phase 04: Remote Skills Registry](./phase-04-remote-skills-registry.md)
- [Phase 06: UI Implementation](./phase-06-ui-implementation.md)

## Overview
**Priority**: P2 | **Status**: pending | **Effort**: 4h

Implement GitHub OAuth authentication for desktop app using PKCE flow. Authentication is **optional** - the app works fully without login. GitHub login enables publishing skills to the community registry.

## Key Insights
- Desktop apps require **OAuth PKCE flow** (no client secret in app binary)
- Use **GitHub OAuth App** (not GitHub App) - simpler for user auth
- Store tokens securely using **Tauri's keyring plugin**
- Token refresh not required - GitHub OAuth tokens don't expire
- Published skills include author's GitHub username for attribution

## App Behavior Without Login
- Browse/search local skills
- Install skills from registry
- Create/edit/delete local skills
- Full CRUD operations

## App Behavior With Login
- All above features PLUS:
- Publish local skills to community registry
- Skills attributed to GitHub username
- View own published skills

## Requirements

### Functional
- F1: GitHub OAuth login via PKCE flow
- F2: Secure token storage in OS keychain
- F3: Auto-check auth status on app launch
- F4: Display user profile (avatar, username)
- F5: Logout and clear stored tokens
- F6: Handle OAuth errors gracefully

### Non-Functional
- NF1: OAuth callback handled within 60s timeout
- NF2: Token stored encrypted in keychain
- NF3: Login flow <5 seconds (excl. user interaction)
- NF4: Works offline after initial auth

## Architecture

### OAuth PKCE Flow (Desktop App)
```
┌─────────────────────────────────────────────────────────────────┐
│                    GitHub OAuth PKCE Flow                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  1. User clicks "Sign in with GitHub"                           │
│     ┌──────────────┐                                            │
│     │ Tauri App    │                                            │
│     └──────┬───────┘                                            │
│            │                                                     │
│  2. Generate code_verifier + code_challenge (PKCE)              │
│            │                                                     │
│  3. Open system browser with auth URL                           │
│            │                                                     │
│            ▼                                                     │
│     ┌──────────────┐                                            │
│     │ GitHub Auth  │  ←─── User logs in + authorizes            │
│     │ (Browser)    │                                            │
│     └──────┬───────┘                                            │
│            │                                                     │
│  4. Redirect to localhost callback with auth code               │
│            │                                                     │
│            ▼                                                     │
│     ┌──────────────┐                                            │
│     │ Local Server │  (Tauri listens on localhost:PORT)         │
│     │ (Callback)   │                                            │
│     └──────┬───────┘                                            │
│            │                                                     │
│  5. Exchange code + code_verifier for access_token              │
│            │                                                     │
│            ▼                                                     │
│     ┌──────────────┐                                            │
│     │ GitHub API   │                                            │
│     │ /oauth/token │                                            │
│     └──────┬───────┘                                            │
│            │                                                     │
│  6. Store token in OS keychain                                  │
│  7. Fetch user profile from /user                               │
│  8. Return user data to frontend                                │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### GitHub OAuth App Setup
```
GitHub Settings > Developer Settings > OAuth Apps > New OAuth App

Application name: AI Skills Aggregator
Homepage URL: https://aiskills.dev
Authorization callback URL: http://127.0.0.1:9876/callback

-> Get Client ID (public, embedded in app)
-> Client Secret NOT used (PKCE flow)
```

## Related Code Files

### Files to Create
- `src-tauri/src/services/auth_service.rs` - OAuth flow implementation
- `src-tauri/src/services/keyring_service.rs` - Secure token storage
- `src-tauri/src/commands/auth.rs` - Auth IPC commands
- `src/hooks/use-auth.ts` - Auth hook for components

### Files to Modify
- `src-tauri/Cargo.toml` - Add tauri-plugin-os, reqwest
- `src-tauri/tauri.conf.json` - Add localhost deep link
- `src-tauri/src/main.rs` - Register auth commands
- `src/lib/api.ts` - Add auth API methods

## Implementation Steps

### 1. Add Rust Dependencies (10 min)

Update `src-tauri/Cargo.toml`:
```toml
[dependencies]
# ... existing
tauri-plugin-os = "2"
keyring = "2"
base64 = "0.21"
sha2 = "0.10"
rand = "0.8"
```

### 2. Create Keyring Service (30 min)

Create `src-tauri/src/services/keyring_service.rs`:
```rust
use keyring::Entry;
use crate::models::AppError;

const SERVICE_NAME: &str = "ai-skills-aggregator";

pub struct KeyringService;

impl KeyringService {
    pub fn store_token(username: &str, token: &str) -> Result<(), AppError> {
        let entry = Entry::new(SERVICE_NAME, username)
            .map_err(|e| AppError::IoError(format!("Keyring error: {}", e)))?;

        entry.set_password(token)
            .map_err(|e| AppError::IoError(format!("Failed to store token: {}", e)))
    }

    pub fn get_token(username: &str) -> Result<String, AppError> {
        let entry = Entry::new(SERVICE_NAME, username)
            .map_err(|e| AppError::IoError(format!("Keyring error: {}", e)))?;

        entry.get_password()
            .map_err(|e| AppError::IoError(format!("Failed to get token: {}", e)))
    }

    pub fn delete_token(username: &str) -> Result<(), AppError> {
        let entry = Entry::new(SERVICE_NAME, username)
            .map_err(|e| AppError::IoError(format!("Keyring error: {}", e)))?;

        entry.delete_credential()
            .map_err(|e| AppError::IoError(format!("Failed to delete token: {}", e)))
    }

    pub fn store_current_user(username: &str) -> Result<(), AppError> {
        let entry = Entry::new(SERVICE_NAME, "current_user")
            .map_err(|e| AppError::IoError(format!("Keyring error: {}", e)))?;

        entry.set_password(username)
            .map_err(|e| AppError::IoError(format!("Failed to store user: {}", e)))
    }

    pub fn get_current_user() -> Result<String, AppError> {
        let entry = Entry::new(SERVICE_NAME, "current_user")
            .map_err(|e| AppError::IoError(format!("Keyring error: {}", e)))?;

        entry.get_password()
            .map_err(|e| AppError::IoError(format!("No stored user: {}", e)))
    }

    pub fn clear_current_user() -> Result<(), AppError> {
        let entry = Entry::new(SERVICE_NAME, "current_user")
            .map_err(|e| AppError::IoError(format!("Keyring error: {}", e)))?;

        entry.delete_credential()
            .map_err(|e| AppError::IoError(format!("Failed to clear user: {}", e)))
    }
}
```

### 3. Create Auth Service (90 min)

Create `src-tauri/src/services/auth_service.rs`:
```rust
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::Rng;
use sha2::{Sha256, Digest};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::models::{User, AppError};
use crate::services::keyring_service::KeyringService;
use crate::services::download_service::DownloadService;

// GitHub OAuth App Client ID (public - safe to embed)
const GITHUB_CLIENT_ID: &str = "YOUR_GITHUB_CLIENT_ID";
const CALLBACK_PORT: u16 = 9876;
const CALLBACK_URL: &str = "http://127.0.0.1:9876/callback";

pub struct AuthService {
    http: DownloadService,
    code_verifier: Arc<Mutex<Option<String>>>,
}

impl AuthService {
    pub fn new() -> Self {
        Self {
            http: DownloadService::new(),
            code_verifier: Arc::new(Mutex::new(None)),
        }
    }

    /// Generate PKCE code verifier and challenge
    fn generate_pkce() -> (String, String) {
        // Generate 32 random bytes for code_verifier
        let mut rng = rand::thread_rng();
        let verifier_bytes: Vec<u8> = (0..32).map(|_| rng.gen::<u8>()).collect();
        let code_verifier = URL_SAFE_NO_PAD.encode(&verifier_bytes);

        // Generate code_challenge = base64url(sha256(code_verifier))
        let mut hasher = Sha256::new();
        hasher.update(code_verifier.as_bytes());
        let hash = hasher.finalize();
        let code_challenge = URL_SAFE_NO_PAD.encode(&hash);

        (code_verifier, code_challenge)
    }

    /// Start OAuth flow - returns authorization URL to open in browser
    pub async fn start_login(&self) -> Result<String, AppError> {
        let (code_verifier, code_challenge) = Self::generate_pkce();

        // Store verifier for later exchange
        let mut verifier = self.code_verifier.lock().await;
        *verifier = Some(code_verifier);

        // Build authorization URL
        let auth_url = format!(
            "https://github.com/login/oauth/authorize?\
            client_id={}&\
            redirect_uri={}&\
            scope=read:user&\
            code_challenge={}&\
            code_challenge_method=S256",
            GITHUB_CLIENT_ID,
            urlencoding::encode(CALLBACK_URL),
            code_challenge
        );

        Ok(auth_url)
    }

    /// Wait for OAuth callback and complete login
    pub async fn wait_for_callback(&self) -> Result<User, AppError> {
        // Start local server to receive callback
        let listener = TcpListener::bind(format!("127.0.0.1:{}", CALLBACK_PORT))
            .await
            .map_err(|e| AppError::IoError(format!("Failed to start callback server: {}", e)))?;

        // Wait for callback (with timeout)
        let (mut socket, _) = tokio::time::timeout(
            std::time::Duration::from_secs(60),
            listener.accept()
        )
        .await
        .map_err(|_| AppError::IoError("Login timeout".into()))?
        .map_err(|e| AppError::IoError(format!("Accept error: {}", e)))?;

        // Read the request
        let mut buffer = [0; 2048];
        socket.read(&mut buffer).await
            .map_err(|e| AppError::IoError(e.to_string()))?;

        let request = String::from_utf8_lossy(&buffer);

        // Extract authorization code from URL
        let code = self.extract_code_from_request(&request)?;

        // Send success response to browser
        let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n\
            <html><body><h1>Login Successful!</h1>\
            <p>You can close this window and return to the app.</p>\
            <script>window.close();</script></body></html>";
        socket.write_all(response.as_bytes()).await.ok();

        // Exchange code for token
        let token = self.exchange_code_for_token(&code).await?;

        // Fetch user profile
        let user = self.fetch_user_profile(&token).await?;

        // Store token securely
        KeyringService::store_token(&user.username, &token)?;
        KeyringService::store_current_user(&user.username)?;

        Ok(user)
    }

    fn extract_code_from_request(&self, request: &str) -> Result<String, AppError> {
        // Parse "GET /callback?code=xxx HTTP/1.1"
        let first_line = request.lines().next().unwrap_or("");
        let url_part = first_line.split_whitespace().nth(1).unwrap_or("");

        if let Some(query_start) = url_part.find('?') {
            let query = &url_part[query_start + 1..];
            for param in query.split('&') {
                if let Some((key, value)) = param.split_once('=') {
                    if key == "code" {
                        return Ok(value.to_string());
                    }
                }
            }
        }

        Err(AppError::ParseError("No authorization code in callback".into()))
    }

    async fn exchange_code_for_token(&self, code: &str) -> Result<String, AppError> {
        let verifier = self.code_verifier.lock().await;
        let code_verifier = verifier.as_ref()
            .ok_or_else(|| AppError::ParseError("Missing code verifier".into()))?;

        let token_url = "https://github.com/login/oauth/access_token";

        let body = format!(
            "client_id={}&\
            code={}&\
            redirect_uri={}&\
            code_verifier={}",
            GITHUB_CLIENT_ID,
            code,
            urlencoding::encode(CALLBACK_URL),
            code_verifier
        );

        let response = self.http.post_form(token_url, &body).await?;

        // Parse response: access_token=xxx&token_type=bearer&scope=read:user
        for param in response.split('&') {
            if let Some((key, value)) = param.split_once('=') {
                if key == "access_token" {
                    return Ok(value.to_string());
                }
            }
        }

        Err(AppError::ParseError("No access token in response".into()))
    }

    async fn fetch_user_profile(&self, token: &str) -> Result<User, AppError> {
        let url = "https://api.github.com/user";
        let response = self.http.get_with_auth(url, token).await?;

        let json: serde_json::Value = serde_json::from_str(&response)
            .map_err(|e| AppError::ParseError(format!("Invalid JSON: {}", e)))?;

        Ok(User {
            id: json["id"].as_i64().unwrap_or(0).to_string(),
            username: json["login"].as_str().unwrap_or("").to_string(),
            display_name: json["name"].as_str().map(String::from),
            avatar_url: json["avatar_url"].as_str().map(String::from),
            access_token: token.to_string(),
            logged_in_at: chrono::Utc::now().timestamp(),
        })
    }

    /// Get currently logged-in user (from stored token)
    pub fn get_current_user(&self) -> Result<User, AppError> {
        let username = KeyringService::get_current_user()?;
        let token = KeyringService::get_token(&username)?;

        // Return cached user data (token is valid)
        Ok(User {
            id: String::new(), // Will be populated if needed
            username: username.clone(),
            display_name: None,
            avatar_url: None,
            access_token: token,
            logged_in_at: 0,
        })
    }

    /// Logout - clear stored tokens
    pub fn logout(&self) -> Result<(), AppError> {
        if let Ok(username) = KeyringService::get_current_user() {
            KeyringService::delete_token(&username).ok();
        }
        KeyringService::clear_current_user()
    }

    /// Check if user is logged in
    pub fn is_logged_in(&self) -> bool {
        KeyringService::get_current_user().is_ok()
    }
}
```

### 4. Create Auth Commands (30 min)

Create `src-tauri/src/commands/auth.rs`:
```rust
use crate::models::User;
use crate::services::auth_service::AuthService;
use tauri::Manager;

#[tauri::command]
pub async fn login(app: tauri::AppHandle) -> Result<User, String> {
    let auth = AuthService::new();

    // Get auth URL
    let auth_url = auth.start_login()
        .await
        .map_err(|e| e.to_string())?;

    // Open in system browser
    tauri::api::shell::open(&app.shell_scope(), &auth_url, None)
        .map_err(|e| format!("Failed to open browser: {}", e))?;

    // Wait for callback
    auth.wait_for_callback()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_current_user() -> Result<Option<User>, String> {
    let auth = AuthService::new();
    match auth.get_current_user() {
        Ok(user) => Ok(Some(user)),
        Err(_) => Ok(None),
    }
}

#[tauri::command]
pub fn logout() -> Result<(), String> {
    let auth = AuthService::new();
    auth.logout().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn is_logged_in() -> bool {
    let auth = AuthService::new();
    auth.is_logged_in()
}
```

### 5. Update main.rs (10 min)

Add to `src-tauri/src/main.rs`:
```rust
mod commands;
use commands::auth;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            // ... existing commands
            auth::login,
            auth::logout,
            auth::get_current_user,
            auth::is_logged_in,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 6. Update Frontend API (15 min)

Add to `src/lib/api.ts`:
```typescript
export const api = {
  // ... existing

  auth: {
    login: () => invoke<User>('login'),
    logout: () => invoke<void>('logout'),
    getCurrentUser: () => invoke<User | null>('get_current_user'),
    isLoggedIn: () => invoke<boolean>('is_logged_in'),
  },

  publish: {
    validate: (skill: Skill) => invoke<string[]>('validate_skill_for_publish', { skill }),
    publish: (skill: Skill, registryUrl: string) =>
      invoke<PublishResponse>('publish_skill', { skill, registryUrl }),
    unpublish: (skillId: string, registryUrl: string) =>
      invoke<void>('unpublish_skill', { skillId, registryUrl }),
  },
};
```

### 7. Create Auth Hook (15 min)

Create `src/hooks/use-auth.ts`:
```typescript
import { useEffect } from 'react';
import { useAuthStore } from '@/stores/auth-store';

export function useAuth() {
  const store = useAuthStore();

  // Check auth status on mount
  useEffect(() => {
    store.checkAuthStatus();
  }, []);

  return {
    user: store.user,
    isLoggedIn: store.authState.type === 'logged_in',
    isLoading: store.isLoading,
    login: store.login,
    logout: store.logout,
  };
}
```

## Todo List
- [ ] Add Rust dependencies (keyring, base64, sha2, rand)
- [ ] Create KeyringService for secure token storage
- [ ] Create AuthService with PKCE implementation
- [ ] Create auth IPC commands
- [ ] Register auth commands in main.rs
- [ ] Update frontend API with auth methods
- [ ] Create useAuth hook
- [ ] Create GitHub OAuth App and get Client ID
- [ ] Test OAuth flow on macOS
- [ ] Test OAuth flow on Windows
- [ ] Test OAuth flow on Linux
- [ ] Test token persistence across app restarts
- [ ] Test logout clears all tokens

## Success Criteria
- [ ] OAuth PKCE flow completes successfully
- [ ] Token stored in OS keychain (not in plaintext)
- [ ] User profile fetched and displayed
- [ ] Auth persists across app restarts
- [ ] Logout clears stored credentials
- [ ] App works fully without login
- [ ] Publish requires login

## Risk Assessment

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Keychain access denied | High | Low | Fallback to encrypted file storage |
| Browser not opening | Medium | Low | Show manual URL to copy |
| Callback timeout | Medium | Medium | Increase timeout, show retry button |
| GitHub API rate limit | Low | Low | Cache user profile locally |

## Security Considerations
- **PKCE required**: No client secret embedded in app
- **Keychain storage**: Tokens encrypted by OS
- **Minimal scopes**: Only `read:user` scope requested
- **No token logging**: Never log access tokens
- **Secure callback**: Localhost-only callback URL
- **HTTPS only**: All GitHub API calls over HTTPS

## GitHub OAuth App Configuration

1. Go to GitHub Settings > Developer Settings > OAuth Apps
2. Click "New OAuth App"
3. Fill in:
   - **Application name**: AI Skills Aggregator
   - **Homepage URL**: https://aiskills.dev
   - **Authorization callback URL**: http://127.0.0.1:9876/callback
4. Click "Register application"
5. Copy **Client ID** (embed in app)
6. **Do NOT use Client Secret** (PKCE flow doesn't need it)

## Next Steps
- Proceed to Phase 08: Testing & Deployment
- Test full publish flow with real GitHub account
- Document OAuth setup for contributors
