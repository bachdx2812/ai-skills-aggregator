use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::fs;

use crate::models::{
    SkillRegistry, RemoteSkill, InstalledSkill, RegistryConfig, SkillUpdate, AppError
};
use crate::services::download_service::DownloadService;

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

    /// Fetch and parse a registry from URL
    pub async fn fetch_registry(&self, config: &RegistryConfig) -> Result<SkillRegistry, AppError> {
        // Check cache first
        let cache_file = self.cache_dir.join(url_to_filename(&config.url));
        if let Ok(cached) = self.read_cache(&cache_file).await {
            // Cache valid for 1 hour
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;

            if now - cached.last_updated < 3600 {
                return Ok(cached);
            }
        }

        // Fetch from remote
        let manifest_url = self.download.convert_github_repo_to_registry(&config.url);
        let content = self.download.fetch_text(&manifest_url).await?;

        let mut registry: SkillRegistry = serde_json::from_str(&content)
            .or_else(|_| serde_yaml::from_str(&content))
            .map_err(|e| AppError::ParseError(format!("Invalid registry format: {}", e)))?;

        registry.url = config.url.clone();
        registry.last_updated = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        // Cache the result
        let _ = self.write_cache(&cache_file, &registry).await;

        Ok(registry)
    }

    /// Install a skill from a remote registry
    pub async fn install_skill(
        &self,
        skill: &RemoteSkill,
        registry_url: &str,
        agent: &str,
    ) -> Result<InstalledSkill, AppError> {
        // Get file path for this agent
        let file_path = self.get_agent_file(skill, agent)
            .ok_or_else(|| AppError::InvalidPath(
                format!("Skill {} doesn't support {}", skill.id, agent)
            ))?;

        // Resolve full URL
        let file_url = if file_path.starts_with("http") {
            file_path.clone()
        } else {
            // Build URL relative to registry
            let base_url = registry_url.trim_end_matches("registry.json").trim_end_matches('/');
            format!("{}/{}", base_url, file_path.trim_start_matches('/'))
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
            installed_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64,
        };

        self.record_installation(&installed).await?;

        Ok(installed)
    }

    /// Uninstall a remote skill
    pub async fn uninstall_skill(&self, skill_id: &str, agent: &str) -> Result<(), AppError> {
        let installed = self.get_installed_skills().await?;

        let skill = installed.iter()
            .find(|s| s.skill_id == skill_id && s.agent == agent)
            .ok_or_else(|| AppError::FileNotFound(
                format!("Skill {} not installed for {}", skill_id, agent)
            ))?;

        // Delete file
        let path = PathBuf::from(&skill.installed_path);
        if path.exists() {
            if path.is_dir() {
                fs::remove_dir_all(&path)
                    .await
                    .map_err(|e| AppError::IoError(e.to_string()))?;
            } else {
                fs::remove_file(&path)
                    .await
                    .map_err(|e| AppError::IoError(e.to_string()))?;
            }
        }

        // Update installed DB
        let remaining: Vec<_> = installed.into_iter()
            .filter(|s| !(s.skill_id == skill_id && s.agent == agent))
            .collect();

        self.save_installed_skills(&remaining).await?;

        Ok(())
    }

    /// Get list of installed skills
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

    /// Check for available updates
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

    fn get_agent_file(&self, skill: &RemoteSkill, agent: &str) -> Option<String> {
        match agent.to_lowercase().as_str() {
            "claude" => skill.files.claude.clone(),
            "cursor" => skill.files.cursor.clone(),
            "continuedev" | "continue" => skill.files.continue_dev.clone(),
            "aider" => skill.files.aider.clone(),
            "windsurf" | "codeium" => skill.files.windsurf.clone(),
            _ => None,
        }
    }

    fn get_install_path(&self, agent: &str, skill_id: &str) -> Result<PathBuf, AppError> {
        let home = dirs::home_dir()
            .ok_or_else(|| AppError::InvalidPath("Cannot find home directory".into()))?;

        // Create skill folder (not just a file)
        let path = match agent.to_lowercase().as_str() {
            "claude" => home.join(".claude").join("skills").join(skill_id).join("skill.md"),
            "cursor" => home.join(".cursor").join("skills").join(skill_id).join("skill.cursorrules"),
            "continuedev" | "continue" => home.join(".continue").join("skills").join(skill_id).join("skill.json"),
            "aider" => home.join(".aider").join("skills").join(skill_id).join("skill.txt"),
            "windsurf" | "codeium" => home.join(".codeium").join("skills").join(skill_id).join("skill.yaml"),
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

impl Default for RegistryService {
    fn default() -> Self {
        Self::new()
    }
}

fn url_to_filename(url: &str) -> String {
    let hash = format!("{:x}", md5::compute(url.as_bytes()));
    format!("{}.json", &hash[..16])
}
