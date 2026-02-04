use std::cmp::Ordering;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::models::{InstalledSkill, RegistryConfig, AppError};
use crate::services::registry_service::RegistryService;

pub struct UpdateService {
    registry: RegistryService,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillUpdate {
    pub skill_id: String,
    pub skill_name: String,
    pub current_version: String,
    pub new_version: String,
    pub agent: String,
    pub registry_url: String,
    pub changelog: Option<String>,
    pub is_major: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCheckResult {
    pub available_updates: Vec<SkillUpdate>,
    pub last_checked: i64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkippedVersion {
    pub skill_id: String,
    pub version: String,
    pub skipped_at: i64,
}

impl UpdateService {
    pub fn new() -> Self {
        Self {
            registry: RegistryService::new(),
        }
    }

    pub async fn check_all_updates(&self) -> UpdateCheckResult {
        let installed = match self.registry.get_installed_skills().await {
            Ok(skills) => skills,
            Err(e) => {
                return UpdateCheckResult {
                    available_updates: vec![],
                    last_checked: Utc::now().timestamp(),
                    error: Some(e.to_string()),
                };
            }
        };

        // Load skipped versions
        let skipped = self.load_skipped_versions().await;

        // Group by registry URL
        let mut by_registry: std::collections::HashMap<String, Vec<&InstalledSkill>> =
            std::collections::HashMap::new();

        for skill in &installed {
            by_registry
                .entry(skill.registry_url.clone())
                .or_default()
                .push(skill);
        }

        let mut updates = Vec::new();

        for (registry_url, skills) in by_registry {
            let config = RegistryConfig {
                url: registry_url.clone(),
                name: "".into(),
                enabled: true,
                auth_token: None,
            };

            if let Ok(registry) = self.registry.fetch_registry(&config).await {
                for installed in skills {
                    if let Some(remote) = registry.skills.iter().find(|s| s.id == installed.skill_id) {
                        // Check if this version is skipped
                        let is_skipped = skipped.iter().any(|s| {
                            s.skill_id == installed.skill_id && s.version == remote.version
                        });

                        if !is_skipped && self.is_newer(&installed.version, &remote.version) {
                            updates.push(SkillUpdate {
                                skill_id: installed.skill_id.clone(),
                                skill_name: remote.name.clone(),
                                current_version: installed.version.clone(),
                                new_version: remote.version.clone(),
                                agent: installed.agent.clone(),
                                registry_url: registry_url.clone(),
                                changelog: None,
                                is_major: self.is_major_update(&installed.version, &remote.version),
                            });
                        }
                    }
                }
            }
        }

        UpdateCheckResult {
            available_updates: updates,
            last_checked: Utc::now().timestamp(),
            error: None,
        }
    }

    pub async fn apply_update(&self, update: &SkillUpdate) -> Result<(), AppError> {
        let config = RegistryConfig {
            url: update.registry_url.clone(),
            name: "".into(),
            enabled: true,
            auth_token: None,
        };

        let registry = self.registry.fetch_registry(&config).await?;

        let remote_skill = registry
            .skills
            .iter()
            .find(|s| s.id == update.skill_id)
            .ok_or_else(|| AppError::FileNotFound(format!("Skill {} not found", update.skill_id)))?;

        // Install (will backup and replace)
        self.registry
            .install_skill(remote_skill, &update.registry_url, &update.agent)
            .await?;

        Ok(())
    }

    pub async fn apply_all_updates(&self, updates: &[SkillUpdate]) -> Vec<Result<(), String>> {
        let mut results = Vec::new();

        for update in updates {
            let result = self.apply_update(update).await.map_err(|e| e.to_string());
            results.push(result);
        }

        results
    }

    pub async fn rollback_skill(&self, skill_id: &str, agent: &str) -> Result<(), AppError> {
        let backup = crate::services::backup_service::BackupService::new();
        let installed = self.registry.get_installed_skills().await?;

        let skill = installed
            .iter()
            .find(|s| s.skill_id == skill_id && s.agent == agent)
            .ok_or_else(|| AppError::FileNotFound(format!("Skill {} not installed", skill_id)))?;

        // Get most recent backup
        let filename = std::path::Path::new(&skill.installed_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        let backups = backup.list_backups(filename).await?;

        if let Some(most_recent) = backups.first() {
            backup.restore_file(&most_recent.path, &skill.installed_path).await?;
            Ok(())
        } else {
            Err(AppError::FileNotFound("No backup available".into()))
        }
    }

    pub async fn skip_version(&self, skill_id: &str, version: &str) -> Result<(), AppError> {
        let config_path = self.get_skipped_versions_path()?;

        let mut skipped = self.load_skipped_versions().await;

        // Check if already skipped
        if skipped.iter().any(|s| s.skill_id == skill_id && s.version == version) {
            return Ok(());
        }

        skipped.push(SkippedVersion {
            skill_id: skill_id.to_string(),
            version: version.to_string(),
            skipped_at: Utc::now().timestamp(),
        });

        let content = serde_json::to_string_pretty(&skipped)
            .map_err(|e| AppError::ParseError(e.to_string()))?;

        if let Some(parent) = config_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| AppError::IoError(e.to_string()))?;
        }

        tokio::fs::write(&config_path, content)
            .await
            .map_err(|e| AppError::IoError(e.to_string()))?;

        Ok(())
    }

    // Internal helpers

    async fn load_skipped_versions(&self) -> Vec<SkippedVersion> {
        let config_path = match self.get_skipped_versions_path() {
            Ok(p) => p,
            Err(_) => return vec![],
        };

        if !config_path.exists() {
            return vec![];
        }

        match tokio::fs::read_to_string(&config_path).await {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => vec![],
        }
    }

    fn get_skipped_versions_path(&self) -> Result<std::path::PathBuf, AppError> {
        let path = dirs::data_local_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("ai-skills-aggregator")
            .join("skipped-versions.json");
        Ok(path)
    }

    fn is_newer(&self, current: &str, available: &str) -> bool {
        self.compare_versions(current, available) == Ordering::Less
    }

    fn is_major_update(&self, current: &str, available: &str) -> bool {
        let curr_major = current.split('.').next().and_then(|s| s.parse::<u32>().ok());
        let avail_major = available.split('.').next().and_then(|s| s.parse::<u32>().ok());

        match (curr_major, avail_major) {
            (Some(c), Some(a)) => a > c,
            _ => false,
        }
    }

    fn compare_versions(&self, current: &str, available: &str) -> Ordering {
        let parse = |v: &str| -> Vec<u32> {
            v.split('.')
                .filter_map(|s| {
                    s.chars()
                        .take_while(|c| c.is_numeric())
                        .collect::<String>()
                        .parse()
                        .ok()
                })
                .collect()
        };

        let curr = parse(current);
        let avail = parse(available);

        for i in 0..3 {
            let c = curr.get(i).copied().unwrap_or(0);
            let a = avail.get(i).copied().unwrap_or(0);
            match c.cmp(&a) {
                Ordering::Equal => continue,
                other => return other,
            }
        }
        Ordering::Equal
    }
}

impl Default for UpdateService {
    fn default() -> Self {
        Self::new()
    }
}
