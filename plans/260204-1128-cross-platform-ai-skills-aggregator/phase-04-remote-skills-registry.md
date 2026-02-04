# Phase 04: Remote Skills Registry

## Context Links
- [Plan Overview](./plan.md)
- [Phase 03: Local Skills Discovery](./phase-03-local-skills-discovery.md)

## Overview
**Priority**: P2 | **Status**: pending | **Effort**: 7h

Implement remote skill repository integration. Support Git-based repos and API-based registries. Enable skill installation from URLs. **NEW**: Implement skill publishing workflow with GitHub authentication.

## Key Insights
- No existing standard for AI skill registries
- GitHub/GitLab repos most accessible format
- Need simple registry manifest (JSON/YAML)
- Consider caching for offline access

## Requirements

### Functional
- F1: Fetch skills from Git repository URLs
- F2: Parse registry manifest files (registry.json)
- F3: Download and install skills to local agent dirs
- F4: Validate skill integrity (checksum optional)
- F5: Track installed skills with source URL
- F6: Support authenticated Git repos (optional)
- F7: **Publish local skills to community registry** (requires GitHub auth)
- F8: **Validate skill metadata before publish**
- F9: **Include author GitHub username in published skills**

### Non-Functional
- NF1: Download timeout: 30s max
- NF2: Cache registry manifests for 1 hour
- NF3: Show download progress in UI
- NF4: Concurrent downloads (max 3)

## Architecture

### Remote Skill Flow
```
┌─────────────────────────────────────────────────────────────────┐
│                      Remote Skills Flow                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────┐  │
│  │ Registry     │───>│ Fetch        │───>│ Parse Registry   │  │
│  │ URL Input    │    │ Manifest     │    │ JSON/YAML        │  │
│  └──────────────┘    └──────────────┘    └──────────────────┘  │
│                                                 │               │
│                                                 ▼               │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────┐  │
│  │ Update Local │<───│ Write to     │<───│ Download Skill   │  │
│  │ Skills DB    │    │ Agent Dir    │    │ File(s)          │  │
│  └──────────────┘    └──────────────┘    └──────────────────┘  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Registry Manifest Schema
```json
{
  "$schema": "https://aiskills.dev/registry-schema.json",
  "version": "1.0",
  "name": "Awesome AI Skills",
  "description": "Community curated AI coding skills",
  "skills": [
    {
      "id": "code-review-expert",
      "name": "Code Review Expert",
      "description": "Thorough code review with security focus",
      "version": "1.2.0",
      "author": "github-username",
      "author_id": "12345678",
      "agents": ["Claude", "Cursor", "Aider"],
      "tags": ["code-review", "security"],
      "files": {
        "claude": "skills/claude/code-review.md",
        "cursor": "skills/cursor/.cursorrules",
        "aider": "skills/aider/code-review.txt"
      },
      "url": "https://github.com/user/repo/tree/main/skills/code-review",
      "published_at": 1706745600
    }
  ]
}
```

### Publishing Flow
```
┌─────────────────────────────────────────────────────────────────┐
│                      Skill Publishing Flow                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────┐   ┌──────────────┐   ┌──────────────────────┐ │
│  │ Select Local │──>│ Validate     │──>│ Check GitHub Auth    │ │
│  │ Skill        │   │ Metadata     │   │ (login if needed)    │ │
│  └──────────────┘   └──────────────┘   └──────────────────────┘ │
│                                                │                 │
│                                                ▼                 │
│  ┌──────────────┐   ┌──────────────┐   ┌──────────────────────┐ │
│  │ Show Success │<──│ Update Local │<──│ POST to Registry API │ │
│  │ + Share Link │   │ Skill Record │   │ (with auth token)    │ │
│  └──────────────┘   └──────────────┘   └──────────────────────┘ │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Related Code Files

### Files to Create
- `src-tauri/src/models/registry.rs` - Registry data models
- `src-tauri/src/models/publish.rs` - Publish request/response models
- `src-tauri/src/services/registry_service.rs` - Registry operations
- `src-tauri/src/services/download_service.rs` - HTTP downloads
- `src-tauri/src/services/publish_service.rs` - Skill publishing logic
- `src-tauri/src/commands/registry.rs` - Registry IPC commands
- `src-tauri/src/commands/publish.rs` - Publish IPC commands
- `src/stores/registry-store.ts` - Frontend registry state
- `src/lib/types.ts` - Add registry types

### Files to Modify
- `src-tauri/src/models/mod.rs` - Export registry + publish models
- `src-tauri/src/services/mod.rs` - Export registry + publish services
- `src-tauri/src/main.rs` - Register registry + publish commands
- `src-tauri/Cargo.toml` - Add reqwest dependency

## Implementation Steps

### 1. Create Registry Models (30 min)

Create `src-tauri/src/models/registry.rs`:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillRegistry {
    pub version: String,
    pub name: String,
    pub description: Option<String>,
    pub url: String,                    // Registry source URL
    pub skills: Vec<RemoteSkill>,
    pub last_updated: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteSkill {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub author: Option<String>,
    pub agents: Vec<String>,            // Supported agents
    pub tags: Vec<String>,
    pub files: SkillFiles,              // Agent-specific files
    pub url: Option<String>,            // Source URL
    pub checksum: Option<String>,       // SHA256 for integrity
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillFiles {
    pub claude: Option<String>,
    pub cursor: Option<String>,
    pub continue_dev: Option<String>,
    pub aider: Option<String>,
    pub windsurf: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledSkill {
    pub skill_id: String,
    pub registry_url: String,
    pub version: String,
    pub installed_path: String,
    pub agent: String,
    pub installed_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    pub url: String,
    pub name: String,
    pub enabled: bool,
    pub auth_token: Option<String>,     // For private repos
}
```

### 2. Create Download Service (45 min)

Create `src-tauri/src/services/download_service.rs`:
```rust
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
}
```

### 3. Create Registry Service (60 min)

Create `src-tauri/src/services/registry_service.rs`:
```rust
use std::collections::HashMap;
use std::path::PathBuf;
use chrono::Utc;
use tokio::fs;

use crate::models::{
    SkillRegistry, RemoteSkill, InstalledSkill, RegistryConfig,
    AgentType, AppError
};
use crate::services::download_service::DownloadService;
use crate::config::default_agent_configs;

pub struct RegistryService {
    download: DownloadService,
    cache_dir: PathBuf,
    installed_db_path: PathBuf,
}

impl RegistryService {
    pub fn new() -> Self {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("ai-skills-aggregator")
            .join("registries");

        let installed_db_path = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("ai-skills-aggregator")
            .join("installed-skills.json");

        Self {
            download: DownloadService::new(),
            cache_dir,
            installed_db_path,
        }
    }

    pub async fn fetch_registry(&self, config: &RegistryConfig) -> Result<SkillRegistry, AppError> {
        // Check cache first
        let cache_file = self.cache_dir.join(url_to_filename(&config.url));
        if let Ok(cached) = self.read_cache(&cache_file).await {
            // Cache valid for 1 hour
            let age = Utc::now().timestamp() - cached.last_updated;
            if age < 3600 {
                return Ok(cached);
            }
        }

        // Fetch from remote
        let manifest_url = self.resolve_manifest_url(&config.url);
        let content = self.download.fetch_text(&manifest_url).await?;

        let mut registry: SkillRegistry = serde_json::from_str(&content)
            .or_else(|_| serde_yaml::from_str(&content))
            .map_err(|e| AppError::ParseError(format!("Invalid registry format: {}", e)))?;

        registry.url = config.url.clone();
        registry.last_updated = Utc::now().timestamp();

        // Cache the result
        self.write_cache(&cache_file, &registry).await?;

        Ok(registry)
    }

    pub async fn install_skill(
        &self,
        skill: &RemoteSkill,
        registry_url: &str,
        agent: &str,
    ) -> Result<InstalledSkill, AppError> {
        // Get file URL for this agent
        let file_path = self.get_agent_file(skill, agent)
            .ok_or_else(|| AppError::InvalidPath(
                format!("Skill {} doesn't support {}", skill.id, agent)
            ))?;

        // Resolve full URL
        let file_url = if file_path.starts_with("http") {
            file_path.clone()
        } else {
            format!("{}/{}", registry_url.trim_end_matches('/'), file_path)
        };

        let raw_url = self.download.convert_github_url_to_raw(&file_url);

        // Determine destination path
        let dest_path = self.get_install_path(agent, &skill.id)?;

        // Download file
        self.download.download_file(&raw_url, &dest_path).await?;

        // Record installation
        let installed = InstalledSkill {
            skill_id: skill.id.clone(),
            registry_url: registry_url.to_string(),
            version: skill.version.clone(),
            installed_path: dest_path.to_string_lossy().to_string(),
            agent: agent.to_string(),
            installed_at: Utc::now().timestamp(),
        };

        self.record_installation(&installed).await?;

        Ok(installed)
    }

    pub async fn uninstall_skill(&self, skill_id: &str, agent: &str) -> Result<(), AppError> {
        let installed = self.get_installed_skills().await?;

        let skill = installed.iter()
            .find(|s| s.skill_id == skill_id && s.agent == agent)
            .ok_or_else(|| AppError::FileNotFound(
                format!("Skill {} not installed for {}", skill_id, agent)
            ))?;

        // Delete file
        fs::remove_file(&skill.installed_path)
            .await
            .map_err(|e| AppError::IoError(e.to_string()))?;

        // Update installed DB
        let remaining: Vec<_> = installed.into_iter()
            .filter(|s| !(s.skill_id == skill_id && s.agent == agent))
            .collect();

        self.save_installed_skills(&remaining).await?;

        Ok(())
    }

    pub async fn get_installed_skills(&self) -> Result<Vec<InstalledSkill>, AppError> {
        if !self.installed_db_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.installed_db_path)
            .await
            .map_err(|e| AppError::IoError(e.to_string()))?;

        serde_json::from_str(&content)
            .map_err(|e| AppError::ParseError(e.to_string()))
    }

    pub async fn check_updates(&self, registry: &SkillRegistry) -> Result<Vec<SkillUpdate>, AppError> {
        let installed = self.get_installed_skills().await?;
        let mut updates = Vec::new();

        for installed_skill in &installed {
            if let Some(remote) = registry.skills.iter().find(|s| s.id == installed_skill.skill_id) {
                if remote.version != installed_skill.version {
                    updates.push(SkillUpdate {
                        skill_id: installed_skill.skill_id.clone(),
                        current_version: installed_skill.version.clone(),
                        new_version: remote.version.clone(),
                        agent: installed_skill.agent.clone(),
                    });
                }
            }
        }

        Ok(updates)
    }

    // Helper methods

    fn resolve_manifest_url(&self, url: &str) -> String {
        if url.contains("github.com") {
            // Convert GitHub repo URL to raw manifest URL
            let raw_url = self.download.convert_github_url_to_raw(url);
            if raw_url.ends_with(".json") || raw_url.ends_with(".yaml") {
                raw_url
            } else {
                format!("{}/registry.json", raw_url.trim_end_matches('/'))
            }
        } else {
            url.to_string()
        }
    }

    fn get_agent_file(&self, skill: &RemoteSkill, agent: &str) -> Option<String> {
        match agent.to_lowercase().as_str() {
            "claude" => skill.files.claude.clone(),
            "cursor" => skill.files.cursor.clone(),
            "continuedev" => skill.files.continue_dev.clone(),
            "aider" => skill.files.aider.clone(),
            "windsurf" => skill.files.windsurf.clone(),
            _ => None,
        }
    }

    fn get_install_path(&self, agent: &str, skill_id: &str) -> Result<PathBuf, AppError> {
        let home = dirs::home_dir()
            .ok_or_else(|| AppError::InvalidPath("Cannot find home directory".into()))?;

        let path = match agent.to_lowercase().as_str() {
            "claude" => home.join(".claude").join("skills").join(format!("{}.md", skill_id)),
            "cursor" => home.join(".cursor").join("skills").join(format!("{}.cursorrules", skill_id)),
            "continuedev" => home.join(".continue").join("skills").join(format!("{}.json", skill_id)),
            "aider" => home.join(".aider").join("skills").join(format!("{}.txt", skill_id)),
            "windsurf" => home.join(".codeium").join("skills").join(format!("{}.yaml", skill_id)),
            _ => return Err(AppError::InvalidPath(format!("Unknown agent: {}", agent))),
        };

        Ok(path)
    }

    async fn read_cache(&self, path: &PathBuf) -> Result<SkillRegistry, AppError> {
        let content = fs::read_to_string(path)
            .await
            .map_err(|e| AppError::IoError(e.to_string()))?;

        serde_json::from_str(&content)
            .map_err(|e| AppError::ParseError(e.to_string()))
    }

    async fn write_cache(&self, path: &PathBuf, registry: &SkillRegistry) -> Result<(), AppError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| AppError::IoError(e.to_string()))?;
        }

        let content = serde_json::to_string_pretty(registry)
            .map_err(|e| AppError::ParseError(e.to_string()))?;

        fs::write(path, content)
            .await
            .map_err(|e| AppError::IoError(e.to_string()))
    }

    async fn record_installation(&self, skill: &InstalledSkill) -> Result<(), AppError> {
        let mut installed = self.get_installed_skills().await.unwrap_or_default();

        // Remove existing entry for same skill+agent
        installed.retain(|s| !(s.skill_id == skill.skill_id && s.agent == skill.agent));
        installed.push(skill.clone());

        self.save_installed_skills(&installed).await
    }

    async fn save_installed_skills(&self, skills: &[InstalledSkill]) -> Result<(), AppError> {
        if let Some(parent) = self.installed_db_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| AppError::IoError(e.to_string()))?;
        }

        let content = serde_json::to_string_pretty(skills)
            .map_err(|e| AppError::ParseError(e.to_string()))?;

        fs::write(&self.installed_db_path, content)
            .await
            .map_err(|e| AppError::IoError(e.to_string()))
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SkillUpdate {
    pub skill_id: String,
    pub current_version: String,
    pub new_version: String,
    pub agent: String,
}

fn url_to_filename(url: &str) -> String {
    let hash = format!("{:x}", md5::compute(url.as_bytes()));
    format!("{}.json", &hash[..16])
}
```

### 4. Create Registry Commands (30 min)

Create `src-tauri/src/commands/registry.rs`:
```rust
use crate::models::{SkillRegistry, RemoteSkill, InstalledSkill, RegistryConfig};
use crate::services::registry_service::{RegistryService, SkillUpdate};

#[tauri::command]
pub async fn fetch_registry(url: String) -> Result<SkillRegistry, String> {
    let service = RegistryService::new();
    let config = RegistryConfig {
        url,
        name: "Custom".into(),
        enabled: true,
        auth_token: None,
    };

    service.fetch_registry(&config)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn install_remote_skill(
    skill: RemoteSkill,
    registry_url: String,
    agent: String,
) -> Result<InstalledSkill, String> {
    let service = RegistryService::new();

    service.install_skill(&skill, &registry_url, &agent)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn uninstall_remote_skill(skill_id: String, agent: String) -> Result<(), String> {
    let service = RegistryService::new();

    service.uninstall_skill(&skill_id, &agent)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_installed_skills() -> Result<Vec<InstalledSkill>, String> {
    let service = RegistryService::new();

    service.get_installed_skills()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn check_skill_updates(registry_url: String) -> Result<Vec<SkillUpdate>, String> {
    let service = RegistryService::new();
    let config = RegistryConfig {
        url: registry_url,
        name: "".into(),
        enabled: true,
        auth_token: None,
    };

    let registry = service.fetch_registry(&config)
        .await
        .map_err(|e| e.to_string())?;

    service.check_updates(&registry)
        .await
        .map_err(|e| e.to_string())
}
```

### 4b. Create Publish Commands (NEW - 30 min)

Create `src-tauri/src/commands/publish.rs`:
```rust
use crate::models::{Skill, PublishRequest, PublishResponse, AppError};
use crate::services::publish_service::PublishService;
use crate::services::auth_service::AuthService;

#[tauri::command]
pub async fn publish_skill(
    skill: Skill,
    registry_url: String,
) -> Result<PublishResponse, String> {
    // Check if user is authenticated
    let auth_service = AuthService::new();
    let user = auth_service.get_current_user()
        .map_err(|e| format!("Not authenticated: {}", e))?;

    // Validate skill has required metadata
    if skill.name.is_empty() {
        return Err("Skill name is required".into());
    }
    if skill.description.is_none() || skill.description.as_ref().unwrap().is_empty() {
        return Err("Skill description is required for publishing".into());
    }

    let service = PublishService::new();
    service.publish(&skill, &registry_url, &user)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn validate_skill_for_publish(skill: Skill) -> Result<Vec<String>, String> {
    let mut errors = Vec::new();

    if skill.name.is_empty() {
        errors.push("Skill name is required".into());
    }
    if skill.name.len() < 3 {
        errors.push("Skill name must be at least 3 characters".into());
    }
    if skill.description.is_none() || skill.description.as_ref().unwrap().is_empty() {
        errors.push("Skill description is required".into());
    }
    if skill.content.len() < 50 {
        errors.push("Skill content must be at least 50 characters".into());
    }
    if skill.tags.is_empty() {
        errors.push("At least one tag is required".into());
    }

    Ok(errors)
}

#[tauri::command]
pub async fn unpublish_skill(
    skill_id: String,
    registry_url: String,
) -> Result<(), String> {
    let auth_service = AuthService::new();
    let user = auth_service.get_current_user()
        .map_err(|e| format!("Not authenticated: {}", e))?;

    let service = PublishService::new();
    service.unpublish(&skill_id, &registry_url, &user)
        .await
        .map_err(|e| e.to_string())
}
```

Create `src-tauri/src/models/publish.rs`:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishRequest {
    pub skill_id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub agents: Vec<String>,
    pub tags: Vec<String>,
    pub content: String,
    pub format: String,
    pub author_username: String,
    pub author_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishResponse {
    pub success: bool,
    pub skill_id: String,
    pub url: String,
    pub message: String,
}
```

Create `src-tauri/src/services/publish_service.rs`:
```rust
use crate::models::{Skill, User, PublishRequest, PublishResponse, AppError};
use crate::services::download_service::DownloadService;

pub struct PublishService {
    client: DownloadService,
}

impl PublishService {
    pub fn new() -> Self {
        Self {
            client: DownloadService::new(),
        }
    }

    pub async fn publish(
        &self,
        skill: &Skill,
        registry_url: &str,
        user: &User,
    ) -> Result<PublishResponse, AppError> {
        let publish_endpoint = format!("{}/api/skills/publish", registry_url.trim_end_matches('/'));

        let request = PublishRequest {
            skill_id: skill.id.clone(),
            name: skill.name.clone(),
            description: skill.description.clone().unwrap_or_default(),
            version: skill.version.clone().unwrap_or("1.0.0".into()),
            agents: vec![format!("{:?}", skill.agent)],
            tags: skill.tags.clone(),
            content: skill.content.clone(),
            format: format!("{:?}", skill.format),
            author_username: user.username.clone(),
            author_id: user.id.clone(),
        };

        // POST with auth token
        let response = self.client.post_json_with_auth(
            &publish_endpoint,
            &request,
            &user.access_token,
        ).await?;

        Ok(response)
    }

    pub async fn unpublish(
        &self,
        skill_id: &str,
        registry_url: &str,
        user: &User,
    ) -> Result<(), AppError> {
        let unpublish_endpoint = format!(
            "{}/api/skills/{}/unpublish",
            registry_url.trim_end_matches('/'),
            skill_id
        );

        self.client.delete_with_auth(&unpublish_endpoint, &user.access_token).await?;
        Ok(())
    }
}
```

### 5. Add Dependencies (10 min)

Update `src-tauri/Cargo.toml`:
```toml
[dependencies]
# ... existing
reqwest = { version = "0.11", features = ["json", "rustls-tls"] }
md5 = "0.7"
```

### 6. Create Frontend Types and Store (30 min)

Add to `src/lib/types.ts`:
```typescript
export interface SkillRegistry {
  version: string;
  name: string;
  description: string | null;
  url: string;
  skills: RemoteSkill[];
  last_updated: number;
}

export interface RemoteSkill {
  id: string;
  name: string;
  description: string | null;
  version: string;
  author: string | null;
  agents: string[];
  tags: string[];
  files: SkillFiles;
  url: string | null;
  checksum: string | null;
}

export interface SkillFiles {
  claude: string | null;
  cursor: string | null;
  continue_dev: string | null;
  aider: string | null;
  windsurf: string | null;
}

export interface InstalledSkill {
  skill_id: string;
  registry_url: string;
  version: string;
  installed_path: string;
  agent: string;
  installed_at: number;
}

export interface SkillUpdate {
  skill_id: string;
  current_version: string;
  new_version: string;
  agent: string;
}
```

Create `src/stores/registry-store.ts`:
```typescript
import { create } from 'zustand';
import type { SkillRegistry, RemoteSkill, InstalledSkill, SkillUpdate } from '@/lib/types';
import { api } from '@/lib/api';

interface RegistryState {
  registries: SkillRegistry[];
  installedSkills: InstalledSkill[];
  updates: SkillUpdate[];
  isLoading: boolean;
  error: string | null;

  fetchRegistry: (url: string) => Promise<void>;
  installSkill: (skill: RemoteSkill, registryUrl: string, agent: string) => Promise<void>;
  uninstallSkill: (skillId: string, agent: string) => Promise<void>;
  loadInstalledSkills: () => Promise<void>;
  checkUpdates: (registryUrl: string) => Promise<void>;
}

export const useRegistryStore = create<RegistryState>((set, get) => ({
  registries: [],
  installedSkills: [],
  updates: [],
  isLoading: false,
  error: null,

  fetchRegistry: async (url) => {
    set({ isLoading: true, error: null });
    try {
      const registry = await api.registry.fetch(url);
      set((state) => ({
        registries: [...state.registries.filter(r => r.url !== url), registry],
        isLoading: false,
      }));
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  installSkill: async (skill, registryUrl, agent) => {
    set({ isLoading: true, error: null });
    try {
      const installed = await api.registry.install(skill, registryUrl, agent);
      set((state) => ({
        installedSkills: [...state.installedSkills, installed],
        isLoading: false,
      }));
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  uninstallSkill: async (skillId, agent) => {
    set({ isLoading: true, error: null });
    try {
      await api.registry.uninstall(skillId, agent);
      set((state) => ({
        installedSkills: state.installedSkills.filter(
          s => !(s.skill_id === skillId && s.agent === agent)
        ),
        isLoading: false,
      }));
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  loadInstalledSkills: async () => {
    try {
      const installed = await api.registry.getInstalled();
      set({ installedSkills: installed });
    } catch (error) {
      set({ error: String(error) });
    }
  },

  checkUpdates: async (registryUrl) => {
    try {
      const updates = await api.registry.checkUpdates(registryUrl);
      set({ updates });
    } catch (error) {
      set({ error: String(error) });
    }
  },
}));
```

Update `src/lib/api.ts`:
```typescript
export const api = {
  // ... existing
  registry: {
    fetch: (url: string) => invoke<SkillRegistry>('fetch_registry', { url }),
    install: (skill: RemoteSkill, registryUrl: string, agent: string) =>
      invoke<InstalledSkill>('install_remote_skill', { skill, registryUrl, agent }),
    uninstall: (skillId: string, agent: string) =>
      invoke<void>('uninstall_remote_skill', { skillId, agent }),
    getInstalled: () => invoke<InstalledSkill[]>('get_installed_skills'),
    checkUpdates: (registryUrl: string) =>
      invoke<SkillUpdate[]>('check_skill_updates', { registryUrl }),
  },
};
```

## Todo List
- [ ] Create registry data models
- [ ] Create publish request/response models
- [ ] Implement download service with timeout
- [ ] Implement registry service (fetch, install, uninstall)
- [ ] Implement publish service (publish, unpublish, validate)
- [ ] Create registry IPC commands
- [ ] Create publish IPC commands
- [ ] Add reqwest and md5 dependencies
- [ ] Create frontend registry types
- [ ] Create Zustand registry store (with publish state)
- [ ] Update API wrappers (incl. publish endpoints)
- [ ] Test with sample GitHub registry
- [ ] Test publish flow with mock registry API
- [ ] Handle network errors gracefully
- [ ] Implement cache invalidation

## Success Criteria
- [ ] Fetch registry from GitHub URL works
- [ ] Install skill downloads and saves correctly
- [ ] Uninstall removes file and updates DB
- [ ] Check updates identifies version mismatches
- [ ] Cache expires after 1 hour
- [ ] Network timeout at 30s
- [ ] GitHub URL conversion works
- [ ] **Publish skill to registry works with GitHub auth**
- [ ] **Published skills include author GitHub username**
- [ ] **Validation errors shown before publish attempt**

## Risk Assessment

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Network failures | High | Medium | Retry logic, offline cache |
| Invalid registry format | Medium | Medium | Validate schema, graceful errors |
| Rate limiting | Medium | Low | Cache, exponential backoff |
| Large file downloads | Low | Low | Stream download, progress UI |

## Security Considerations
- Validate URLs before fetching
- No arbitrary code execution from remote
- HTTPS only for downloads
- Verify checksums if provided
- Sanitize file paths
- **Verify GitHub auth token before accepting publish requests**
- **Rate limit publish requests per user**
- **Validate published skill content (no malicious patterns)**

## Next Steps
- Proceed to Phase 05: Skills CRUD Operations
- Build skill editor UI
- Implement skill creation workflow
