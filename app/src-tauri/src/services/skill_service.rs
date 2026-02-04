use std::path::{Path, PathBuf};
use std::fs;
use glob::glob;

use crate::models::{AgentConfig, AgentType, Skill, SkillFile, SkillFormat, AppError};

pub struct SkillService;

impl SkillService {
    /// Scan all enabled agents for skills
    pub fn scan_all_skills(configs: &[AgentConfig]) -> Result<Vec<Skill>, AppError> {
        let mut all_skills = Vec::new();

        for config in configs.iter().filter(|c| c.enabled) {
            match Self::scan_agent_skills(config) {
                Ok(skills) => all_skills.extend(skills),
                Err(e) => {
                    log::warn!("Failed to scan skills for {:?}: {}", config.agent, e);
                }
            }
        }

        Ok(all_skills)
    }

    /// Scan skills for a specific agent
    pub fn scan_agent_skills(config: &AgentConfig) -> Result<Vec<Skill>, AppError> {
        let mut skills = Vec::new();

        // Scan skills directory - each subdirectory is a skill
        if let Some(skills_dir) = &config.skills_dir {
            let skills_path = Path::new(skills_dir);
            if skills_path.exists() {
                if let Ok(entries) = fs::read_dir(skills_path) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            // Each directory is a skill folder
                            if let Ok(skill) = Self::parse_skill_folder(&path, &config.agent) {
                                skills.push(skill);
                            }
                        }
                        // Skip loose files in skills directory - only folders are skills
                    }
                }
            }
        }

        // Also scan for special config files (CLAUDE.md, .cursorrules, etc.)
        let config_path = Path::new(&config.config_dir);
        if config_path.exists() {
            for pattern in &config.file_patterns {
                // Only scan patterns that are direct files (not in skills subdir)
                if pattern.contains("skills/") {
                    continue;
                }
                let full_pattern = config_path.join(pattern);
                let pattern_str = full_pattern.to_string_lossy();

                if let Ok(entries) = glob(&pattern_str) {
                    for entry in entries.flatten() {
                        if entry.is_file() {
                            // Skip if this file is inside the skills directory
                            if let Some(skills_dir) = &config.skills_dir {
                                if entry.to_string_lossy().contains(skills_dir) {
                                    continue;
                                }
                            }
                            if let Ok(skill) = Self::parse_single_file(&entry, &config.agent) {
                                skills.push(skill);
                            }
                        }
                    }
                }
            }
        }

        Ok(skills)
    }

    /// Parse a skill folder into a Skill struct
    pub fn parse_skill_folder(folder_path: &Path, agent: &AgentType) -> Result<Skill, AppError> {
        let folder_name = folder_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Scan all files in the folder
        let mut files = Vec::new();
        let mut entry_found = false;

        if let Ok(entries) = fs::read_dir(folder_path) {
            for entry in entries.flatten() {
                let file_path = entry.path();
                if file_path.is_file() {
                    let file_name = file_path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown");

                    let extension = file_path.extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("");

                    let format = SkillFormat::from_extension(extension);

                    // Determine if this is the entry file
                    let is_entry = !entry_found && (
                        file_name == "skill.md" ||
                        file_name == "index.md" ||
                        file_name == "README.md" ||
                        file_name == format!("{}.md", folder_name)
                    );

                    if is_entry {
                        entry_found = true;
                    }

                    let size = fs::metadata(&file_path)
                        .map(|m| m.len())
                        .unwrap_or(0);

                    files.push(SkillFile {
                        name: file_name.to_string(),
                        file_path: file_path.to_string_lossy().to_string(),
                        format,
                        is_entry,
                        size,
                    });
                }
            }
        }

        // Also scan subdirectories (references/, scripts/, etc.)
        if let Ok(entries) = fs::read_dir(folder_path) {
            for entry in entries.flatten() {
                let sub_path = entry.path();
                if sub_path.is_dir() {
                    Self::scan_subdirectory(&sub_path, &mut files)?;
                }
            }
        }

        // Sort files: entry first, then alphabetically
        files.sort_by(|a, b| {
            if a.is_entry && !b.is_entry {
                std::cmp::Ordering::Less
            } else if !a.is_entry && b.is_entry {
                std::cmp::Ordering::Greater
            } else {
                a.name.cmp(&b.name)
            }
        });

        let mut skill = Skill::new_folder(
            folder_name,
            folder_path.to_string_lossy().to_string(),
            agent.clone(),
            files,
        );

        // Extract description from entry file
        if let Some(entry_path) = &skill.entry_file {
            if let Ok(content) = fs::read_to_string(entry_path) {
                skill.description = Self::extract_description(&content);
            }
        }

        // Get folder metadata for timestamps
        if let Ok(metadata) = fs::metadata(folder_path) {
            if let Ok(modified) = metadata.modified() {
                skill.updated_at = modified
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(skill.updated_at);
            }
            if let Ok(created) = metadata.created() {
                skill.created_at = created
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(skill.created_at);
            }
        }

        Ok(skill)
    }

    /// Scan a subdirectory and add files to the list
    fn scan_subdirectory(dir_path: &Path, files: &mut Vec<SkillFile>) -> Result<(), AppError> {
        if let Ok(entries) = fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                let file_path = entry.path();
                if file_path.is_file() {
                    let file_name = file_path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown");

                    let extension = file_path.extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("");

                    let format = SkillFormat::from_extension(extension);
                    let size = fs::metadata(&file_path)
                        .map(|m| m.len())
                        .unwrap_or(0);

                    files.push(SkillFile {
                        name: file_name.to_string(),
                        file_path: file_path.to_string_lossy().to_string(),
                        format,
                        is_entry: false,
                        size,
                    });
                }
            }
        }
        Ok(())
    }

    /// Parse a single file as a skill (for config files like CLAUDE.md)
    pub fn parse_single_file(path: &Path, agent: &AgentType) -> Result<Skill, AppError> {
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let extension = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let format = SkillFormat::from_extension(extension);
        let name = file_name.rsplit('.').last()
            .unwrap_or(file_name)
            .to_string();

        let mut skill = Skill::new_single_file(
            name,
            path.to_string_lossy().to_string(),
            agent.clone(),
            format,
        );

        // Extract description from content
        if let Ok(content) = fs::read_to_string(path) {
            skill.description = Self::extract_description(&content);
        }

        // Get file metadata for timestamps
        if let Ok(metadata) = fs::metadata(path) {
            if let Ok(modified) = metadata.modified() {
                skill.updated_at = modified
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(skill.updated_at);
            }
            if let Ok(created) = metadata.created() {
                skill.created_at = created
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(skill.created_at);
            }
        }

        Ok(skill)
    }

    /// Extract description from content
    fn extract_description(content: &str) -> Option<String> {
        // Look for first paragraph or header
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            // Skip headers
            if trimmed.starts_with('#') {
                continue;
            }
            // Return first non-empty, non-header line
            return Some(trimmed.chars().take(200).collect());
        }
        None
    }

    /// Read skill content from file
    pub fn read_content(file_path: &str) -> Result<String, AppError> {
        fs::read_to_string(file_path)
            .map_err(|e| AppError::IoError(e.to_string()))
    }

    /// Get files in a skill folder
    pub fn get_skill_files(folder_path: &str) -> Result<Vec<SkillFile>, AppError> {
        let path = Path::new(folder_path);
        if !path.exists() {
            return Err(AppError::FileNotFound(folder_path.to_string()));
        }

        let mut files = Vec::new();

        if path.is_dir() {
            // Recursive scan
            Self::scan_directory_recursive(path, &mut files)?;
        } else {
            // Single file
            let file_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");
            let extension = path.extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");
            let format = SkillFormat::from_extension(extension);
            let size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);

            files.push(SkillFile {
                name: file_name.to_string(),
                file_path: folder_path.to_string(),
                format,
                is_entry: true,
                size,
            });
        }

        Ok(files)
    }

    /// Recursively scan a directory for files
    fn scan_directory_recursive(dir_path: &Path, files: &mut Vec<SkillFile>) -> Result<(), AppError> {
        if let Ok(entries) = fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let file_name = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown");
                    let extension = path.extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("");
                    let format = SkillFormat::from_extension(extension);
                    let size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

                    files.push(SkillFile {
                        name: file_name.to_string(),
                        file_path: path.to_string_lossy().to_string(),
                        format,
                        is_entry: false,
                        size,
                    });
                } else if path.is_dir() {
                    Self::scan_directory_recursive(&path, files)?;
                }
            }
        }
        Ok(())
    }

    /// Create a new skill folder
    pub fn create_skill(
        agent: &AgentType,
        name: &str,
        content: &str,
        description: Option<&str>,
        tags: Option<Vec<String>>,
        configs: &[AgentConfig],
    ) -> Result<Skill, AppError> {
        let config = configs.iter()
            .find(|c| &c.agent == agent)
            .ok_or_else(|| AppError::NotFound(format!("Agent config not found: {:?}", agent)))?;

        // Determine the target directory
        let target_dir = config.skills_dir.as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(&config.config_dir));

        // Create skill folder
        let folder_name = name.to_lowercase().replace(' ', "-");
        let skill_folder = target_dir.join(&folder_name);

        if skill_folder.exists() {
            return Err(AppError::AlreadyExists(skill_folder.to_string_lossy().to_string()));
        }

        fs::create_dir_all(&skill_folder)?;

        // Create entry file (skill.md)
        let entry_file = skill_folder.join("skill.md");
        let mut file_content = content.to_string();

        // Add description as header if provided
        if let Some(desc) = description {
            if !content.starts_with('#') {
                file_content = format!("# {}\n\n{}\n\n{}", name, desc, content);
            }
        }

        fs::write(&entry_file, &file_content)?;

        // Parse and return the skill
        let mut skill = Self::parse_skill_folder(&skill_folder, agent)?;
        skill.description = description.map(String::from);
        skill.tags = tags.unwrap_or_default();
        skill.is_local = true;

        Ok(skill)
    }

    /// Update skill file content
    pub fn update_skill(file_path: &str, content: &str) -> Result<Skill, AppError> {
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(AppError::FileNotFound(file_path.to_string()));
        }

        fs::write(path, content)?;

        // Return updated skill - find the parent skill folder
        let parent = path.parent()
            .ok_or_else(|| AppError::InvalidPath("Cannot get parent directory".to_string()))?;

        let agent = Self::detect_agent_from_path(parent);

        if parent.is_dir() && parent.file_name().map(|n| n.to_str().unwrap_or("")).unwrap_or("") != ".claude" {
            Self::parse_skill_folder(parent, &agent)
        } else {
            Self::parse_single_file(path, &agent)
        }
    }

    /// Delete skill (folder or file)
    pub fn delete_skill(path: &str) -> Result<(), AppError> {
        let p = Path::new(path);
        if !p.exists() {
            return Err(AppError::FileNotFound(path.to_string()));
        }

        if p.is_dir() {
            fs::remove_dir_all(p)?;
        } else {
            fs::remove_file(p)?;
        }

        Ok(())
    }

    /// Duplicate a skill folder
    pub fn duplicate_skill(folder_path: &str, new_name: &str) -> Result<Skill, AppError> {
        let path = Path::new(folder_path);
        if !path.exists() {
            return Err(AppError::FileNotFound(folder_path.to_string()));
        }

        let parent = path.parent()
            .ok_or_else(|| AppError::InvalidPath("Cannot get parent directory".to_string()))?;

        let new_folder_name = new_name.to_lowercase().replace(' ', "-");
        let new_path = parent.join(&new_folder_name);

        if new_path.exists() {
            return Err(AppError::AlreadyExists(new_path.to_string_lossy().to_string()));
        }

        // Copy folder recursively
        Self::copy_dir_recursive(path, &new_path)?;

        let agent = Self::detect_agent_from_path(&new_path);
        Self::parse_skill_folder(&new_path, &agent)
    }

    /// Copy directory recursively
    fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), AppError> {
        fs::create_dir_all(dst)?;

        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if src_path.is_dir() {
                Self::copy_dir_recursive(&src_path, &dst_path)?;
            } else {
                fs::copy(&src_path, &dst_path)?;
            }
        }

        Ok(())
    }

    /// Detect agent type from file path
    fn detect_agent_from_path(path: &Path) -> AgentType {
        let path_str = path.to_string_lossy().to_lowercase();

        if path_str.contains(".claude") {
            AgentType::Claude
        } else if path_str.contains(".cursor") {
            AgentType::Cursor
        } else if path_str.contains(".continue") {
            AgentType::ContinueDev
        } else if path_str.contains(".aider") {
            AgentType::Aider
        } else if path_str.contains(".codeium") {
            AgentType::Windsurf
        } else {
            AgentType::Custom("Unknown".to_string())
        }
    }

    /// Create a new file in a skill folder
    pub fn create_file(
        skill_folder: &str,
        file_name: &str,
        content: Option<&str>,
    ) -> Result<SkillFile, AppError> {
        let folder_path = Path::new(skill_folder);
        if !folder_path.exists() {
            return Err(AppError::FileNotFound(skill_folder.to_string()));
        }

        let file_path = folder_path.join(file_name);

        if file_path.exists() {
            return Err(AppError::AlreadyExists(file_path.to_string_lossy().to_string()));
        }

        let file_content = content.unwrap_or("");
        fs::write(&file_path, file_content)?;

        let extension = file_path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        let format = SkillFormat::from_extension(extension);
        let size = file_content.len() as u64;

        Ok(SkillFile {
            name: file_name.to_string(),
            file_path: file_path.to_string_lossy().to_string(),
            format,
            is_entry: false,
            size,
        })
    }

    /// Delete a single file
    pub fn delete_file(file_path: &str) -> Result<(), AppError> {
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(AppError::FileNotFound(file_path.to_string()));
        }

        fs::remove_file(path)?;
        Ok(())
    }
}
