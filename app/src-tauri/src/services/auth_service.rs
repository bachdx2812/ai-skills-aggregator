use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::{thread_rng, Rng};
use sha2::{Sha256, Digest};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::models::{User, AppError};
use crate::services::keyring_service::KeyringService;
use crate::services::download_service::DownloadService;

// GitHub OAuth App Client ID (public - safe to embed)
// TODO: Replace with your actual GitHub OAuth App Client ID
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
        let mut rng = thread_rng();
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
            <html><body style=\"font-family: system-ui; text-align: center; padding: 50px;\">\
            <h1>✅ Login Successful!</h1>\
            <p>You can close this window and return to the app.</p>\
            <script>setTimeout(() => window.close(), 1500);</script></body></html>";
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

        // Check for error
        if url_part.contains("error=") {
            return Err(AppError::ParseError("Authorization denied by user".into()));
        }

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

        // Try JSON format first (Accept: application/json)
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response) {
            if let Some(token) = json.get("access_token").and_then(|t| t.as_str()) {
                return Ok(token.to_string());
            }
            if let Some(error) = json.get("error_description").and_then(|e| e.as_str()) {
                return Err(AppError::ParseError(format!("OAuth error: {}", error)));
            }
        }

        // Fallback to form-encoded format
        for param in response.split('&') {
            if let Some((key, value)) = param.split_once('=') {
                if key == "access_token" {
                    return Ok(value.to_string());
                }
            }
        }

        Err(AppError::ParseError(format!("No access token in response: {}", response)))
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
    pub async fn get_current_user(&self) -> Result<User, AppError> {
        let username = KeyringService::get_current_user()?;
        let token = KeyringService::get_token(&username)?;

        // Fetch fresh profile
        self.fetch_user_profile(&token).await
    }

    /// Get current user without network call (from cache)
    pub fn get_current_user_cached(&self) -> Result<User, AppError> {
        let username = KeyringService::get_current_user()?;
        let token = KeyringService::get_token(&username)?;

        Ok(User {
            id: String::new(),
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

impl Default for AuthService {
    fn default() -> Self {
        Self::new()
    }
}
