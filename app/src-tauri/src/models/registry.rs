use serde::{Deserialize, Serialize};

/// Remote skill registry manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillRegistry {
    pub version: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub url: String,
    pub skills: Vec<RemoteSkill>,
    #[serde(default)]
    pub last_updated: i64,
}

/// A skill available in a remote registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteSkill {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub author: Option<String>,
    #[serde(default)]
    pub agents: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub files: SkillFiles,
    pub url: Option<String>,
    pub checksum: Option<String>,
}

/// Agent-specific file paths within the registry
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillFiles {
    pub claude: Option<String>,
    pub cursor: Option<String>,
    #[serde(rename = "continue_dev")]
    pub continue_dev: Option<String>,
    pub aider: Option<String>,
    pub windsurf: Option<String>,
}

/// Record of an installed remote skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledSkill {
    pub skill_id: String,
    pub registry_url: String,
    pub version: String,
    pub installed_path: String,
    pub agent: String,
    pub installed_at: i64,
}

/// Registry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    pub url: String,
    pub name: String,
    pub enabled: bool,
    pub auth_token: Option<String>,
}

/// Skill update information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillUpdate {
    pub skill_id: String,
    pub current_version: String,
    pub new_version: String,
    pub agent: String,
}

/// User information from GitHub OAuth
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub access_token: String,
    pub logged_in_at: i64,
}

/// Publish request payload
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

/// Publish response from registry API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishResponse {
    pub success: bool,
    pub skill_id: String,
    pub url: String,
    pub message: String,
}
