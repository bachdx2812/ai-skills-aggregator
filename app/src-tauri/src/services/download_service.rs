use reqwest::Client;
use std::path::PathBuf;
use std::time::Duration;
use tokio::fs;
use crate::models::AppError;

pub struct DownloadService {
    client: Client,
}

impl DownloadService {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("AI-Skills-Aggregator/1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    /// Fetch text content from a URL
    pub async fn fetch_text(&self, url: &str) -> Result<String, AppError> {
        let response = self.client
            .get(url)
            .send()
            .await
            .map_err(|e| AppError::IoError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::IoError(format!(
                "HTTP {} for {}",
                response.status(),
                url
            )));
        }

        response
            .text()
            .await
            .map_err(|e| AppError::IoError(format!("Failed to read response: {}", e)))
    }

    /// Download a file to a destination path
    pub async fn download_file(&self, url: &str, dest: &PathBuf) -> Result<(), AppError> {
        let response = self.client
            .get(url)
            .send()
            .await
            .map_err(|e| AppError::IoError(format!("Download failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::IoError(format!(
                "HTTP {} downloading {}",
                response.status(),
                url
            )));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| AppError::IoError(format!("Failed to read bytes: {}", e)))?;

        // Ensure parent directory exists
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| AppError::IoError(format!("Failed to create dir: {}", e)))?;
        }

        fs::write(dest, bytes)
            .await
            .map_err(|e| AppError::IoError(format!("Failed to write file: {}", e)))?;

        Ok(())
    }

    /// Convert GitHub blob URLs to raw content URLs
    pub fn convert_github_url_to_raw(&self, url: &str) -> String {
        // Convert GitHub blob URLs to raw content URLs
        // https://github.com/user/repo/blob/main/file.md
        // -> https://raw.githubusercontent.com/user/repo/main/file.md
        if url.contains("github.com") && url.contains("/blob/") {
            url.replace("github.com", "raw.githubusercontent.com")
               .replace("/blob/", "/")
        } else {
            url.to_string()
        }
    }

    /// POST form data (for OAuth token exchange)
    pub async fn post_form(&self, url: &str, body: &str) -> Result<String, AppError> {
        let response = self.client
            .post(url)
            .header("Accept", "application/json")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body.to_string())
            .send()
            .await
            .map_err(|e| AppError::IoError(format!("POST failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::IoError(format!(
                "HTTP {} for POST {}",
                response.status(),
                url
            )));
        }

        response
            .text()
            .await
            .map_err(|e| AppError::IoError(format!("Failed to read response: {}", e)))
    }

    /// GET with Authorization header
    pub async fn get_with_auth(&self, url: &str, token: &str) -> Result<String, AppError> {
        let response = self.client
            .get(url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| AppError::IoError(format!("GET failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::IoError(format!(
                "HTTP {} for {}",
                response.status(),
                url
            )));
        }

        response
            .text()
            .await
            .map_err(|e| AppError::IoError(format!("Failed to read response: {}", e)))
    }

    /// Convert GitHub tree/repo URLs to raw registry.json URL
    pub fn convert_github_repo_to_registry(&self, url: &str) -> String {
        if url.contains("github.com") {
            // Handle various GitHub URL formats
            let clean_url = url
                .replace("/tree/", "/")
                .replace("/blob/", "/");

            // If URL ends with repo or branch, append registry.json
            if !clean_url.ends_with(".json") && !clean_url.ends_with(".yaml") {
                let raw_base = clean_url
                    .replace("github.com", "raw.githubusercontent.com");

                // Add main branch if not specified
                if raw_base.matches('/').count() == 4 {
                    format!("{}/main/registry.json", raw_base)
                } else {
                    format!("{}/registry.json", raw_base.trim_end_matches('/'))
                }
            } else {
                self.convert_github_url_to_raw(&clean_url)
            }
        } else {
            url.to_string()
        }
    }
}

impl Default for DownloadService {
    fn default() -> Self {
        Self::new()
    }
}
