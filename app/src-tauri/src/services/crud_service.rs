use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::fs;
use serde::Serialize;

use crate::models::{Skill, SkillFile, AgentType, SkillFormat, AppError};
use crate::services::backup_service::BackupService;
use crate::services::template_service::TemplateService;

pub struct CrudService {
    backup: BackupService,
}

impl CrudService {
    pub fn new() -> Self {
        Self {
            backup: BackupService::new(),
        }
    }

    /// Create a new skill folder with an initial file
    pub async fn create_skill(
        &self,
        name: &str,
        description: Option<String>,
        tags: Vec<String>,
        agent: &AgentType,
        format: &SkillFormat,
        content: Option<String>,
    ) -> Result<Skill, AppError> {
        // Validate name
        if name.len() < 2 {
            return Err(AppError::InvalidPath("Skill name must be at least 2 characters".into()));
        }

        // Get agent skills directory
        let skills_dir = self.get_agent_skills_dir(agent)?;

        // Create skill folder
        let sanitized_name = self.sanitize_filename(name);
        let skill_folder = skills_dir.join(&sanitized_name);

        // Check if folder already exists
        if skill_folder.exists() {
            return Err(AppError::InvalidPath(format!("Skill '{}' already exists", name)));
        }

        // Create the folder
        fs::create_dir_all(&skill_folder)
            .await
            .map_err(|e| AppError::IoError(e.to_string()))?;

        // Create initial file (skill.md or appropriate entry file)
        let file_name = format!("skill.{}", format.extension());
        let file_path = skill_folder.join(&file_name);

        let skill_content = content.unwrap_or_else(|| {
            TemplateService::get_template(agent, format)
        });

        fs::write(&file_path, &skill_content)
            .await
            .map_err(|e| AppError::IoError(e.to_string()))?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let file = SkillFile {
            name: file_name,
            file_path: file_path.to_string_lossy().to_string(),
            format: format.clone(),
            is_entry: true,
            size: skill_content.len() as u64,
        };

        Ok(Skill {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            description,
            folder_path: skill_folder.to_string_lossy().to_string(),
            agent: agent.clone(),
            files: vec![file.clone()],
            entry_file: Some(file.file_path),
            tags,
            version: Some("1.0.0".to_string()),
            author: None,
            is_local: true,
            is_folder: true,
            file_count: 1,
            created_at: now,
            updated_at: now,
        })
    }

    /// Create a new file within an existing skill folder
    pub async fn create_file(
        &self,
        skill_folder: &str,
        file_name: &str,
        format: &SkillFormat,
        content: Option<String>,
    ) -> Result<SkillFile, AppError> {
        let folder_path = PathBuf::from(skill_folder);
        if !folder_path.exists() {
            return Err(AppError::FileNotFound(skill_folder.to_string()));
        }

        // Sanitize and build file path
        let sanitized_name = self.sanitize_filename(file_name);
        let full_name = if sanitized_name.contains('.') {
            sanitized_name.clone()
        } else {
            format!("{}.{}", sanitized_name, format.extension())
        };

        let file_path = folder_path.join(&full_name);

        // Check if file already exists
        if file_path.exists() {
            return Err(AppError::InvalidPath(format!("File '{}' already exists", full_name)));
        }

        // Get content from template or provided
        let file_content = content.unwrap_or_default();

        // Write file
        fs::write(&file_path, &file_content)
            .await
            .map_err(|e| AppError::IoError(e.to_string()))?;

        Ok(SkillFile {
            name: full_name,
            file_path: file_path.to_string_lossy().to_string(),
            format: format.clone(),
            is_entry: false,
            size: file_content.len() as u64,
        })
    }

    /// Read file content
    pub async fn read_content(&self, file_path: &str) -> Result<String, AppError> {
        fs::read_to_string(file_path)
            .await
            .map_err(|e| AppError::IoError(e.to_string()))
    }

    /// Update file content
    pub async fn update_content(
        &self,
        file_path: &str,
        content: &str,
        create_backup: bool,
    ) -> Result<(), AppError> {
        // Validate content
        self.validate_content(content)?;

        let path = PathBuf::from(file_path);

        // Create backup if requested and file exists
        if create_backup && path.exists() {
            self.backup.backup_file(file_path).await?;
        }

        // Atomic write: write to temp file then rename
        let temp_path = path.with_extension("tmp");

        fs::write(&temp_path, content)
            .await
            .map_err(|e| AppError::IoError(e.to_string()))?;

        fs::rename(&temp_path, &path)
            .await
            .map_err(|e| AppError::IoError(e.to_string()))?;

        Ok(())
    }

    /// Delete a skill (entire folder)
    pub async fn delete_skill(&self, folder_path: &str) -> Result<(), AppError> {
        let path = PathBuf::from(folder_path);

        if !path.exists() {
            return Err(AppError::FileNotFound(folder_path.to_string()));
        }

        // Backup before delete
        self.backup.backup_folder(folder_path).await?;

        // Delete folder
        if path.is_dir() {
            fs::remove_dir_all(&path)
                .await
                .map_err(|e| AppError::IoError(e.to_string()))?;
        } else {
            fs::remove_file(&path)
                .await
                .map_err(|e| AppError::IoError(e.to_string()))?;
        }

        Ok(())
    }

    /// Delete a single file within a skill
    pub async fn delete_file(&self, file_path: &str) -> Result<(), AppError> {
        let path = PathBuf::from(file_path);

        if !path.exists() {
            return Err(AppError::FileNotFound(file_path.to_string()));
        }

        // Backup before delete
        self.backup.backup_file(file_path).await?;

        // Delete file
        fs::remove_file(&path)
            .await
            .map_err(|e| AppError::IoError(e.to_string()))?;

        Ok(())
    }

    /// Duplicate a skill folder
    pub async fn duplicate_skill(
        &self,
        source_path: &str,
        new_name: &str,
    ) -> Result<String, AppError> {
        let source = PathBuf::from(source_path);
        if !source.exists() {
            return Err(AppError::FileNotFound(source_path.to_string()));
        }

        let parent = source.parent()
            .ok_or_else(|| AppError::InvalidPath("Cannot find parent directory".into()))?;

        let sanitized_name = self.sanitize_filename(new_name);
        let dest = parent.join(&sanitized_name);

        if dest.exists() {
            return Err(AppError::InvalidPath(format!("'{}' already exists", new_name)));
        }

        // Copy folder recursively
        self.copy_dir_recursive(&source, &dest).await?;

        Ok(dest.to_string_lossy().to_string())
    }

    /// Rename a skill folder
    pub async fn rename_skill(
        &self,
        folder_path: &str,
        new_name: &str,
    ) -> Result<String, AppError> {
        let source = PathBuf::from(folder_path);
        if !source.exists() {
            return Err(AppError::FileNotFound(folder_path.to_string()));
        }

        let parent = source.parent()
            .ok_or_else(|| AppError::InvalidPath("Cannot find parent directory".into()))?;

        let sanitized_name = self.sanitize_filename(new_name);
        let dest = parent.join(&sanitized_name);

        if dest.exists() {
            return Err(AppError::InvalidPath(format!("'{}' already exists", new_name)));
        }

        fs::rename(&source, &dest)
            .await
            .map_err(|e| AppError::IoError(e.to_string()))?;

        Ok(dest.to_string_lossy().to_string())
    }

    /// Export skill content
    pub async fn export_skill(&self, file_path: &str) -> Result<ExportData, AppError> {
        let content = self.read_content(file_path).await?;
        let path = PathBuf::from(file_path);

        let filename = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("skill.md")
            .to_string();

        Ok(ExportData { filename, content })
    }

    // Helper methods

    fn get_agent_skills_dir(&self, agent: &AgentType) -> Result<PathBuf, AppError> {
        let home = dirs::home_dir()
            .ok_or_else(|| AppError::InvalidPath("Cannot find home directory".into()))?;

        let path = match agent {
            AgentType::Claude => home.join(".claude").join("skills"),
            AgentType::Cursor => home.join(".cursor").join("skills"),
            AgentType::ContinueDev => home.join(".continue").join("skills"),
            AgentType::Aider => home.join(".aider").join("prompts"),
            AgentType::Windsurf => home.join(".codeium").join("skills"),
            AgentType::Custom(name) => home.join(format!(".{}", name.to_lowercase())).join("skills"),
        };

        Ok(path)
    }

    fn sanitize_filename(&self, name: &str) -> String {
        name.chars()
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
            .collect::<String>()
            .to_lowercase()
    }

    fn validate_content(&self, content: &str) -> Result<(), AppError> {
        // Check max size (1MB)
        if content.len() > 1_000_000 {
            return Err(AppError::InvalidPath("Content too large (max 1MB)".into()));
        }

        // Check for binary content
        if content.bytes().any(|b| b == 0) {
            return Err(AppError::InvalidPath("Binary content not allowed".into()));
        }

        Ok(())
    }

    async fn copy_dir_recursive(&self, src: &PathBuf, dst: &PathBuf) -> Result<(), AppError> {
        fs::create_dir_all(dst)
            .await
            .map_err(|e| AppError::IoError(e.to_string()))?;

        let mut entries = fs::read_dir(src)
            .await
            .map_err(|e| AppError::IoError(e.to_string()))?;

        while let Some(entry) = entries.next_entry()
            .await
            .map_err(|e| AppError::IoError(e.to_string()))?
        {
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if src_path.is_dir() {
                Box::pin(self.copy_dir_recursive(&src_path, &dst_path)).await?;
            } else {
                fs::copy(&src_path, &dst_path)
                    .await
                    .map_err(|e| AppError::IoError(e.to_string()))?;
            }
        }

        Ok(())
    }
}

impl Default for CrudService {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ExportData {
    pub filename: String,
    pub content: String,
}
